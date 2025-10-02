use clap::Parser;
use dotenv::dotenv;
use gerrit_faster::changes::serve as changes;
use gerrit_faster::context::ServiceContext;
use gerrit_faster::discord::serve as discord;
use gerrit_faster::webserver::serve as webserver;
use tokio;
use tracing::{Level, info};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to run the webserver on
    #[clap(short, long, default_value_t = 3000)]
    port: u16,
    /// Debug mode
    #[clap(short, long, default_value_t = false)]
    debug: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(if args.debug {
            Level::DEBUG
        } else {
            Level::INFO
        })
        .init();

    dotenv().ok();

    let context = ServiceContext::new();
    info!("Service ServiceContext: {:?}", context);

    tokio::join!(
        webserver::serve(context.clone(), args.port),
        discord::serve(context.clone()),
        changes::serve(context.clone())
    );
}
