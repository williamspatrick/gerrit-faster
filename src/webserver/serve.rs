use crate::changes as Changes;
use crate::changes::report as ChangeReport;
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
        .route("/review-status/{id}", get(review_status))
        .route("/report", get(report_overall))
        .route("/report/{*project}", get(report_project))
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

async fn report_overall(
    Extension(context): Extension<ServiceContext>,
) -> Html<String> {
    let report_text = ChangeReport::report(&context, None);

    Html(format!(
        "<html><head><title>Overall Status</title></head><body><h1>Overall Status</h1><pre style=\"font-family: monospace;\">{}</pre></body></html>",
        report_text
    ))
}

async fn report_project(
    Path(project): Path<String>,
    Extension(context): Extension<ServiceContext>,
) -> Html<String> {
    // Remove the leading slash that comes with the wildcard pattern
    let project = project.strip_prefix('/').unwrap_or(&project).to_string();

    let report_text = ChangeReport::report(&context, Some(project.clone()));

    Html(format!(
        "<html><head><title>Project {}</title></head><body><h1>Project {}</h1><pre style=\"font-family: monospace;\">{}</pre></body></html>",
        project, project, report_text
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
