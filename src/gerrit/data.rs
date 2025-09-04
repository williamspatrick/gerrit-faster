use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ChangeInfoRaw {
    pub change_id: String,
    pub project: String,
    pub branch: String,

    pub subject: String,

    pub created: String,
    pub updated: String,

    pub status: String,
    pub work_in_progress: Option<bool>,
}

#[derive(Debug)]
pub struct ChangeInfo {
    pub change_id: String,
    pub project: String,
    pub branch: String,

    pub subject: String,

    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,

    pub status: String,
    pub work_in_progress: bool,
}

impl From<ChangeInfoRaw> for ChangeInfo {
    fn from(raw: ChangeInfoRaw) -> Self {
        ChangeInfo {
            change_id: raw.change_id,
            project: raw.project,
            branch: raw.branch,
            subject: raw.subject,
            created: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::parse_from_str(&raw.created, "%Y-%m-%d %H:%M:%S%.f").unwrap(),
                Utc,
            ),
            updated: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::parse_from_str(&raw.updated, "%Y-%m-%d %H:%M:%S%.f").unwrap(),
                Utc,
            ),
            status: raw.status,
            work_in_progress: raw.work_in_progress.unwrap_or(false),
        }
    }
}
