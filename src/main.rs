use dotenv::dotenv;
use gerrit_faster::context::ServiceContext;
use gerrit_faster::discord::serve as discord;
use gerrit_faster::gerrit::connection::GerritConnection;
use gerrit_faster::webserver::serve as webserver;
use tokio;
use tracing::info;
use tracing_subscriber;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    let context = ServiceContext::new();
    info!("Service ServiceContext: {:?}", context);

    tokio::join!(
        webserver::serve(context.clone()),
        discord::serve(context.clone()),
        async {
            let changes = context.gerrit.all_open_changes().await.unwrap();

            for change in &changes {
                info!("Change: {:?}", change);
            }

            info!("Total Changes: {:?}", changes.len());
        }
    );
}
