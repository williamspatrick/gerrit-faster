use askama::Template;

#[derive(Template)]
#[template(path = "overall.html")]
pub struct OverallTemplate {
    pub report_text: String,
    pub changes_text: String,
}

#[derive(Template)]
#[template(path = "repo.html")]
pub struct RepoTemplate {
    pub report_text: String,
}

#[derive(Template)]
#[template(path = "project.html")]
pub struct ProjectTemplate {
    pub project: String,
    pub report_text: String,
    pub changes_text: String,
}

#[derive(Template)]
#[template(path = "user.html")]
pub struct UserTemplate {
    pub username: String,
    pub report_text: String,
    pub changes_text: String,
}

#[derive(Template)]
#[template(path = "change.html")]
pub struct ChangeTemplate {
    pub change_id: String,
    pub review_status: String,
    pub gerrit_url: String,
}

#[derive(Template)]
#[template(path = "change_not_found.html")]
pub struct ChangeNotFoundTemplate {
    pub change_id: String,
}

#[derive(Template)]
#[template(path = "root.html")]
pub struct RootTemplate;
