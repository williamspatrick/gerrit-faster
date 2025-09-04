use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ChangeInfo {
    pub change_id: String,
    pub project: String,
    pub branch: String,

    pub subject: String,

    pub created: String,
    pub updated: String,

    pub status: String,
    pub work_in_progress: Option<bool>,
}
