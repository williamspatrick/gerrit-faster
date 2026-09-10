use crate::changes::container::Change;
use crate::changes::filter::should_include_change;
use crate::changes::report::{
    TimeInterval, changes_by_owner_time, report_by_time,
};
use crate::changes::status::{NextStepOwner, ReviewState};
use crate::context::ServiceContext;
use chrono::{DateTime, Datelike, Days, TimeZone, Utc, Weekday};
use chrono_tz::America::Detroit;
use poise::serenity_prelude as serenity;
use rand::prelude::*;
use rand::rng;
use tracing::{error, info, warn};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, ServiceContext, Error>;

// Constants for community review change selection
const TOTAL_CHANGES_TO_SELECT: usize = 10;
const RECENT_CHANGES_TO_SELECT: usize = 8;

// Give a report of outstanding changes.
#[poise::command(slash_command, prefix_command, rename = "obmc-report")]
async fn report(
    ctx: Context<'_>,
    #[description = "Project"] project: Option<String>,
) -> Result<(), Error> {
    let service = ctx.data().clone();

    let report = report_by_time(&service, project.clone(), None);

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
    let change: Option<Change>;
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
                    // Apply the community filter
                    if should_include_change(&change.change) {
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
    }

    // Randomly shuffle both groups
    let mut rng = rng();
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

// Next 10am on a weekday in America/Detroit, returned as UTC.
// Weekends are skipped: notifications fire Monday-Friday only.
fn next_10am_detroit(now: DateTime<Utc>) -> DateTime<Utc> {
    let local_now = now.with_timezone(&Detroit);
    let mut day = local_now.date_naive();

    loop {
        // Skip Saturday and Sunday.
        if !matches!(day.weekday(), Weekday::Sat | Weekday::Sun) {
            let candidate = Detroit
                .from_local_datetime(&day.and_hms_opt(10, 0, 0).unwrap());
            // 10am never falls in a DST transition, so a single
            // mapping is expected; fall back to earliest just in
            // case.
            if let Some(candidate) = candidate.earliest() {
                let candidate_utc = candidate.with_timezone(&Utc);
                if candidate_utc > now {
                    return candidate_utc;
                }
            }
        }
        day = day.checked_add_days(Days::new(1)).unwrap();
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
            warn!(
                "DISCORD_REVIEW_CHANNEL_ID not set, community review reminders disabled."
            );
            return;
        }
    };

    loop {
        // Sleep until the next 10am in Detroit (handles EST/EDT
        // offsets and DST automatically via chrono-tz).
        let now = Utc::now();
        let next = next_10am_detroit(now);
        let seconds_until_next = (next - now).num_seconds().max(1) as u64 + 1;

        info!(
            "Next Discord review reminder at {} Detroit time ({} UTC).",
            next.with_timezone(&Detroit),
            next,
        );

        tokio::time::sleep(tokio::time::Duration::from_secs(
            seconds_until_next,
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono::Timelike;

    #[test]
    fn morning_edt_schedules_today() {
        // 09:00 EDT = 13:00 UTC, before 10am cutoff.
        let now = Utc.with_ymd_and_hms(2026, 9, 10, 13, 0, 0).unwrap();
        let next = next_10am_detroit(now);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 10, 14, 0, 0).unwrap());
    }

    #[test]
    fn afternoon_edt_schedules_tomorrow() {
        // 11:00 EDT = 15:00 UTC, after 10am cutoff.
        let now = Utc.with_ymd_and_hms(2026, 9, 10, 15, 0, 0).unwrap();
        let next = next_10am_detroit(now);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 11, 14, 0, 0).unwrap());
    }

    #[test]
    fn morning_est_schedules_today() {
        // 08:00 EST = 13:00 UTC in January.
        let now = Utc.with_ymd_and_hms(2026, 1, 15, 13, 0, 0).unwrap();
        let next = next_10am_detroit(now);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 1, 15, 15, 0, 0).unwrap());
    }

    #[test]
    fn friday_afternoon_schedules_monday() {
        // Friday 11:00 EDT = 15:00 UTC, after 10am cutoff.
        // Next weekday 10am is Monday 10:00 EDT = 14:00 UTC.
        let now = Utc.with_ymd_and_hms(2026, 9, 11, 15, 0, 0).unwrap();
        let next = next_10am_detroit(now);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 14, 14, 0, 0).unwrap());
    }

    #[test]
    fn saturday_schedules_monday() {
        // Saturday 10:00 EDT = 14:00 UTC.
        let now = Utc.with_ymd_and_hms(2026, 9, 12, 14, 0, 0).unwrap();
        let next = next_10am_detroit(now);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 14, 14, 0, 0).unwrap());
    }

    #[test]
    fn sunday_schedules_monday() {
        // Sunday 10:00 EDT = 14:00 UTC.
        let now = Utc.with_ymd_and_hms(2026, 9, 13, 14, 0, 0).unwrap();
        let next = next_10am_detroit(now);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 14, 14, 0, 0).unwrap());
    }

    #[test]
    fn result_is_always_10am_detroit() {
        for (y, m, d, h) in [
            (2026, 9, 10, 0),
            (2026, 9, 10, 15),
            (2026, 9, 11, 15),
            (2026, 9, 12, 14),
            (2026, 9, 13, 14),
            (2026, 1, 15, 13),
        ] {
            let now = Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap();
            let local = next_10am_detroit(now).with_timezone(&Detroit);
            assert_eq!(
                (local.hour(), local.minute()),
                (10, 0),
                "now={} gave local={}",
                now,
                local,
            );
            assert!(
                !matches!(
                    local.weekday(),
                    chrono::Weekday::Sat | chrono::Weekday::Sun
                ),
                "now={} scheduled on weekend: {}",
                now,
                local,
            );
        }
    }
}
