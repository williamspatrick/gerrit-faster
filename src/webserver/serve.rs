use crate::changes as Changes;
use crate::context::ServiceContext;
use crate::gerrit::connection::GerritConnection;
use axum::{
    Extension, Router,
    extract::Path,
    response::{Html, IntoResponse},
    routing::get,
};
use tower::ServiceBuilder;

pub async fn serve(context: ServiceContext) {
    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/review-status/:id", get(review_status))
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

async fn review_status(
    Path(change_id): Path<String>,
    Extension(context): Extension<ServiceContext>,
) -> impl IntoResponse {
    let change: Option<Changes::container::Change>;
    {
        let changes = &context.lock().unwrap().changes;

        let id = change_id.parse::<u64>();
        change = match id {
            Ok(i) => changes.get(i),
            _ => changes.get_by_change_id(&change_id),
        }
    }

    if let Some(change) = change {
        Html(std::format!(
            "<html><body><h1>Change {}</h1><p>Review Status: {:?}</p></body></html>",
            change_id,
            change.review_state
        )).into_response()
    } else {
        (axum::http::StatusCode::NOT_FOUND, Html(std::format!(
            "<html><body><h1>Change Not Found</h1><p>Could not find change: {}</p></body></html>",
            change_id
        ))).into_response()
    }
}
