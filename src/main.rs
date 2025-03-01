use dotenv::dotenv;
use gerrit_faster::webserver::serve as webserver;
use tokio;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv().ok();

    tokio::join!(webserver::serve());
}
