use dotenv::dotenv;
use gerrit_faster::discord::serve as discord;
use gerrit_faster::webserver::serve as webserver;
use tokio;
use tracing_subscriber;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    tokio::join!(webserver::serve(), discord::serve());
}
