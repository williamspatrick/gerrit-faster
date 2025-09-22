use crate::changes as Changes;
use crate::changes::container::Change;
use crate::changes::report::{
    self as ChangeReport, TimeInterval, changes_by_owner_time,
};
use crate::changes::status::{NextStepOwner, ReviewState};
use crate::context::ServiceContext;
use chrono::Timelike;
use poise::serenity_prelude as serenity;
use rand::prelude::*;
use tracing::{error, info};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, ServiceContext, Error>;

// Constants for community review change selection
const TOTAL_CHANGES_TO_SELECT: usize = 5;
const RECENT_CHANGES_TO_SELECT: usize = 4;

// Give a report of outstanding changes.
#[poise::command(slash_command, prefix_command, rename = "obmc-report")]
async fn report(
    ctx: Context<'_>,
    #[description = "Project"] project: Option<String>,
) -> Result<(), Error> {
    let service = ctx.data().clone();

    let report = ChangeReport::report_by_time(&service, project.clone(), None);

    let response = if let Some(ref project_name) = project {
        format!("Project {}:\n```\n{}\n```", project_name, report)
    } else {
        format!("Overall Status:\n```\n{}\n```", report)
    };
    ctx.say(response).await?;

    Ok(())
}

// Get the the review status of a Gerrit change.
#[poise::command(slash_command, prefix_command, rename = "obmc-review-status")]
async fn review_status(
    ctx: Context<'_>,
    #[description = "Change ID"] change_id: String,
) -> Result<(), Error> {
    let change: Option<Changes::container::Change>;
    {
        let changes = &ctx.data().lock().unwrap().changes;

        let id = change_id.parse::<u64>();
        change = match id {
            Ok(i) => changes.get(i),
            _ => changes.get_by_change_id(&change_id),
        }
    }

    let response = if change.is_some() {
        format!(
            "Change {} is {:?}.",
            change_id,
            change.unwrap().review_state
        )
    } else {
        format!("Could not find change: {}", change_id)
    };
    ctx.say(response).await?;
    Ok(())
}

// Get changes that need community review, selecting up to RECENT_CHANGES_TO_SELECT changes that are
// under 24 hours or under 72 hours old, and then selecting additional changes
// from the over 72 hours group to make a total of TOTAL_CHANGES_TO_SELECT changes.
// Returns a tuple of (selected_changes, total_community_review_count)
async fn get_community_review_changes(
    context: &ServiceContext,
) -> (Vec<Change>, usize) {
    // Use existing changes_by_owner_time function to get changes
    let changes_by_time = changes_by_owner_time(context, None, None);

    // Get the lock on the context to access changes
    let ctx = context.lock().unwrap();

    // Collect changes in CommunityReview state, separating into recent and older groups
    let mut recent_changes = Vec::new();
    let mut older_changes = Vec::new();
    let mut total_community_review_count = 0;

    // Process all time intervals in a single iteration
    for time_interval in [
        TimeInterval::Under24Hours,
        TimeInterval::Under72Hours,
        TimeInterval::Under2Weeks,
        TimeInterval::Under8Weeks,
        TimeInterval::Over8Weeks,
    ] {
        let change_ids = changes_by_time
            .get_changes(time_interval, NextStepOwner::Community);

        for id in change_ids {
            if let Some(change) = ctx.changes.get(id) {
                // Double-check that the change is actually in CommunityReview state
                if matches!(change.review_state, ReviewState::CommunityReview) {
                    total_community_review_count += 1;
                    // Categorize changes based on time interval
                    match time_interval {
                        TimeInterval::Under24Hours
                        | TimeInterval::Under72Hours => {
                            recent_changes.push(change);
                        }
                        _ => {
                            older_changes.push(change);
                        }
                    }
                }
            }
        }
    }

    // Randomly shuffle both groups
    let mut rng = thread_rng();
    recent_changes.shuffle(&mut rng);
    older_changes.shuffle(&mut rng);

    // Select up to RECENT_CHANGES_TO_SELECT recent changes
    let recent_count =
        std::cmp::min(RECENT_CHANGES_TO_SELECT, recent_changes.len());
    let mut selected_changes: Vec<Change> =
        recent_changes[..recent_count].to_vec();

    // Select additional changes from older group to make a total of TOTAL_CHANGES_TO_SELECT
    let additional_count = TOTAL_CHANGES_TO_SELECT - selected_changes.len();
    let older_count = std::cmp::min(additional_count, older_changes.len());
    selected_changes.extend(older_changes[..older_count].to_vec());

    (selected_changes, total_community_review_count)
}

// Send community review reminder to Discord
async fn send_community_review_reminder(
    context: &ServiceContext,
    http: &serenity::Http,
    channel_id: u64,
) {
    let (changes, total_count) = get_community_review_changes(context).await;

    if changes.is_empty() {
        return;
    }

    let mut embed = serenity::CreateEmbed::new()
        .title("Review Reminder")
        .description("Want to help with reviews? Here are a few...")
        .color((38, 139, 210)); // Blue color

    for change in &changes {
        let change_url = format!(
            "https://gerrit.openbmc.org/c/{}/+/{}",
            change.change.project, change.change.id_number
        );

        // Calculate waiting time
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(change.review_state_updated);
        let waiting_time = format_duration(duration);

        let field_value = format!(
            "[{}]({}) (+{}/-{})",
            change.change.subject,
            change_url,
            change.change.insertions,
            change.change.deletions,
        );

        embed = embed.field(
            format!("{} - waiting {}", change.change.project, waiting_time),
            field_value,
            false, // Inline: false means each field will be on its own line
        );
    }

    // Add footer with count of additional changes
    let additional_count = total_count.saturating_sub(changes.len());
    if additional_count > 0 {
        let footer = serenity::CreateEmbedFooter::new(format!(
            "And there are {} more...",
            additional_count
        ));
        embed = embed.footer(footer);
    }

    // Add webserver link if WEBSERVER_HOSTNAME is set
    if let Ok(hostname) = std::env::var("WEBSERVER_HOSTNAME") {
        embed = embed.url(format!("https://{}/bot/report", hostname));
    }

    let channel_id = serenity::ChannelId::new(channel_id);
    if let Err(e) = channel_id
        .send_message(http, serenity::CreateMessage::new().add_embed(embed))
        .await
    {
        error!("Failed to send message to Discord channel: {}", e);
    }
}

// Format duration as simple time string like "1 hour" or "3 days"
fn format_duration(duration: chrono::Duration) -> String {
    let hours = duration.num_hours();
    let days = duration.num_days();

    if days > 0 {
        if days == 1 {
            "1 day".to_string()
        } else {
            format!("{} days", days)
        }
    } else if hours > 0 {
        if hours == 1 {
            "1 hour".to_string()
        } else {
            format!("{} hours", hours)
        }
    } else {
        "less than 1 hour".to_string()
    }
}

// Periodic task for sending community review reminders
async fn community_review_reminder_task(
    context: ServiceContext,
    http: &serenity::Http,
) {
    // Get the channel ID from environment variable or exit if not set
    let channel_id = match std::env::var("DISCORD_REVIEW_CHANNEL_ID") {
        Ok(id) => match id.parse::<u64>() {
            Ok(parsed_id) => parsed_id,
            Err(_) => {
                error!("Invalid DISCORD_REVIEW_CHANNEL_ID: {}", id);
                return;
            }
        },
        Err(_) => {
            // Channel ID not set, exit the task
            info!(
                "DISCORD_REVIEW_CHANNEL_ID not set, community review reminders disabled."
            );
            return;
        }
    };

    loop {
        // Calculate delay until next hour using wall clock time
        let now = chrono::Utc::now();
        let next_hour = now
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
            + chrono::Duration::hours(2);

        let duration = next_hour.signed_duration_since(now);
        let seconds_until_next_hour = duration.num_seconds() as u64 + 1;

        // Sleep until next hour
        tokio::time::sleep(tokio::time::Duration::from_secs(
            seconds_until_next_hour,
        ))
        .await;

        // Send reminder
        send_community_review_reminder(&context, http, channel_id).await;
    }
}

pub async fn serve(context: ServiceContext) {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![report(), review_status()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(
                    ctx,
                    &framework.options().commands,
                )
                .await?;

                // Set Nickname in each guild.
                for guild in _ready.guilds.iter() {
                    guild.id.edit_nickname(ctx, Some("openbmc-bot")).await?;
                }

                // Clone context and http for the periodic task
                let context_clone = context.clone();
                let http = ctx.http.clone();

                // Start the periodic task for community review reminders
                tokio::spawn(async move {
                    community_review_reminder_task(
                        context_clone,
                        http.as_ref(),
                    )
                    .await;
                });

                Ok(context)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
