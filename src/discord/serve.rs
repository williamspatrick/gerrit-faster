use crate::changes as Changes;
use crate::changes::report as ChangeReport;
use crate::context::ServiceContext;
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, ServiceContext, Error>;

// Give a report of outstanding changes.
#[poise::command(slash_command, prefix_command, rename = "obmc-report")]
async fn report(
    ctx: Context<'_>,
    #[description = "Project"] project: Option<String>,
) -> Result<(), Error> {
    let service = ctx.data().clone();

    let report = ChangeReport::report(&service, project.clone(), None);

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

                Ok(context)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
