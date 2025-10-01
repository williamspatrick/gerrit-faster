use crate::changes::filter::should_include_change;
use crate::changes::report::{self as ChangeReport, TimeInterval};
use crate::changes::{self as Changes, status::NextStepOwner};
use crate::context::ServiceContext;
use crate::webserver::templates::*;
use askama::Template;
use axum::{
    Extension, Router,
    extract::Path,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use tower::ServiceBuilder;

pub async fn serve(context: ServiceContext, port: u16) {
    // build our application with a route
    let app = Router::new()
        .route("/bot", get(root))
        .route("/bot/report", get(report_overall))
        .route("/bot/report-by-repo", get(report_repo))
        .route("/bot/report/{*project}", get(report_project))
        .route("/bot/review-status/{id}", get(review_status))
        .route("/bot/style.css", get(css))
        .route("/bot/user/{id}", get(report_user))
        .layer(ServiceBuilder::new().layer(Extension(context)));

    // run it
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn root(Extension(_context): Extension<ServiceContext>) -> Html<String> {
    let template = RootTemplate;
    Html(template.render().unwrap())
}

fn list_of_changes(
    changes: &ChangeReport::ChangesByOwnerAndTime,
    context: &ServiceContext,
    include_author: bool,
) -> String {
    let mut result = String::new();
    for owner in [
        NextStepOwner::Author,
        NextStepOwner::Community,
        NextStepOwner::Maintainer,
    ] {
        if owner == NextStepOwner::Author && !include_author {
            continue;
        }

        // Check if this owner has any changes to display
        let mut owner_has_changes = false;
        for interval in [
            TimeInterval::Under24Hours,
            TimeInterval::Under72Hours,
            TimeInterval::Under2Weeks,
            TimeInterval::Under8Weeks,
            TimeInterval::Over8Weeks,
        ] {
            let local_changes = changes.get_changes(interval, owner);
            if !local_changes.is_empty() {
                owner_has_changes = true;
                break;
            }
        }

        if !owner_has_changes {
            continue;
        }

        let owner_str = format!("{:?}", owner);
        let section_id = owner_str.to_lowercase();
        result += &format!(
            r#"
<h2>
    <button class="twisty" onclick="toggleSection('{}')" aria-label="Toggle section"></button> {}
</h2>
<div id="{}-section">
"#,
            section_id, owner_str, section_id
        );

        for interval in [
            TimeInterval::Under24Hours,
            TimeInterval::Under72Hours,
            TimeInterval::Under2Weeks,
            TimeInterval::Under8Weeks,
            TimeInterval::Over8Weeks,
        ] {
            let local_changes = changes.get_changes(interval, owner);
            if local_changes.is_empty() {
                continue;
            }

            result += &format!(
                "<div class=\"interval-section\">\n<h3 class=\"interval-header\">{}</h3>\n",
                interval.to_string()
            );
            result += "<div class=\"card-container\">\n";

            for change in local_changes {
                let change_data_opt =
                    context.lock().unwrap().changes.get(change);
                if let Some(change_data) = change_data_opt {
                    result += &format!(
                        "<div class=\"change-card{}\">\n",
                        if should_include_change(&change_data.change) {
                            ""
                        } else {
                            " company-change"
                        }
                    );
                    result += &format!(
                        "<div class=\"project-name\">{}</div>\n",
                        change_data.change.project
                    );

                    if owner == NextStepOwner::Author {
                        result += &format!(
                            "<div class=\"review-state\">{:?}</div>\n",
                            change_data.review_state
                        );
                    }

                    result += &format!(
                        "<div class=\"subject\">{}</div>\n",
                        change_data.change.subject
                    );
                    result += &format!(
                        "<a class=\"gerrit-link\" href=\"https://gerrit.openbmc.org/c/{}/+/{}\">View in Gerrit</a>\n",
                        change_data.change.project,
                        change_data.change.id_number
                    );

                    // Add insertions/deletions box
                    result += &format!(
                        "<div class=\"change-stats\"><span class=\"insertions\">+{}</span> / <span class=\"deletions\">-{}</span></div>\n",
                        change_data.change.insertions,
                        change_data.change.deletions
                    );

                    result += "</div>\n";
                }
            }
            result += "</div>\n"; // Close interval-container
            result += "</div>\n"; // Close interval-section
        }
        result += "</div>\n"; // Close owner section
    }

    result
}

async fn report_overall(
    Extension(context): Extension<ServiceContext>,
) -> Html<String> {
    let changes = ChangeReport::changes_by_owner_time(&context, None, None);
    let report_text = ChangeReport::report_by_owner_time(&changes);
    let changes_text = list_of_changes(&changes, &context, false);

    let template = OverallTemplate {
        report_text,
        changes_text,
    };
    Html(template.render().unwrap())
}

async fn report_repo(
    Extension(context): Extension<ServiceContext>,
) -> Html<String> {
    let report_text = ChangeReport::report_by_repo(
        &context,
        None,
        Some(|repo: &str| {
            format!("<a href=\"/bot/report/{}\">{}</a>", repo, repo)
        }),
    );

    let template = RepoTemplate { report_text };
    Html(template.render().unwrap())
}

async fn report_project(
    Path(project): Path<String>,
    Extension(context): Extension<ServiceContext>,
) -> Html<String> {
    // Remove the leading slash that comes with the wildcard pattern
    let project = project.strip_prefix('/').unwrap_or(&project).to_string();

    let changes = ChangeReport::changes_by_owner_time(
        &context,
        Some(project.clone()),
        None,
    );
    let report_text = ChangeReport::report_by_owner_time(&changes);
    let changes_text = list_of_changes(&changes, &context, false);

    let template = ProjectTemplate {
        project,
        report_text,
        changes_text,
    };
    Html(template.render().unwrap())
}

async fn report_user(
    Path(username): Path<String>,
    Extension(context): Extension<ServiceContext>,
) -> Html<String> {
    // Remove the leading slash that comes with the wildcard pattern
    let username = username.strip_prefix('/').unwrap_or(&username).to_string();

    let changes = ChangeReport::changes_by_owner_time(
        &context,
        None,
        Some(username.clone()),
    );
    let report_text = ChangeReport::report_by_owner_time(&changes);
    let changes_text = list_of_changes(&changes, &context, true);

    let template = UserTemplate {
        username,
        report_text,
        changes_text,
    };
    Html(template.render().unwrap())
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
        let template = ChangeTemplate {
            change_id,
            review_status: format!("{:?}", change.review_state),
        };
        Html(template.render().unwrap()).into_response()
    } else {
        let template = ChangeNotFoundTemplate { change_id };
        (
            axum::http::StatusCode::NOT_FOUND,
            Html(template.render().unwrap()),
        )
            .into_response()
    }
}

async fn css() -> Response {
    let css_content = include_str!("../../templates/style.css");
    Response::builder()
        .header("Content-Type", "text/css")
        .body(axum::body::Body::from(css_content))
        .unwrap()
}
