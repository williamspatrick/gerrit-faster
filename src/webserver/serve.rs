use crate::context::ServiceContext;
use crate::gerrit::connection::GerritConnection;
use axum::{response::Html, routing::get, Extension, Router};
use tower::ServiceBuilder;

pub async fn serve(context: ServiceContext) {
    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .layer(ServiceBuilder::new().layer(Extension(context)));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn root(Extension(context): Extension<ServiceContext>) -> Html<String> {
    Html(std::format!(
        "Connecting to Gerrit as '{}'!",
        context.get_gerrit().get_username()
    ))
}
