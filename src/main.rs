use clap::Parser;
use dotenv::dotenv;
use gerrit_faster::changes::serve as changes;
use gerrit_faster::context::ServiceContext;
use gerrit_faster::discord::serve as discord;
use gerrit_faster::webserver::serve as webserver;
use tokio;
use tracing::info;
use tracing_subscriber;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to run the webserver on
    #[clap(short, long, default_value_t = 3000)]
    port: u16,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt::init();

    dotenv().ok();

    let context = ServiceContext::new();
    info!("Service ServiceContext: {:?}", context);

    tokio::join!(
        webserver::serve(context.clone(), args.port),
        discord::serve(context.clone()),
        changes::serve(context.clone())
    );
}
