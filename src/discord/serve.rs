use crate::changes as Changes;
use crate::context::ServiceContext;
use crate::gerrit::connection::GerritConnection;
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, ServiceContext, Error>;

// Get the status of the service.
#[poise::command(slash_command, prefix_command, rename = "obmc-service-status")]
async fn service_status(ctx: Context<'_>) -> Result<(), Error> {
    let service = ctx.data();

    let response =
        format!("Running as '{}'", service.get_gerrit().get_username());
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
            commands: vec![service_status(), review_status()],
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
