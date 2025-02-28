use axum::{response::Html, routing::get, Router};
use dotenv::dotenv;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv().ok();

    // build our application with a route
    let app = Router::new().route("/", get(handler));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<String> {
    let username = std::env::var("GERRIT_USERNAME").expect("USERNAME must be set");
    Html(std::format!("Hello, {}!", username))
}
