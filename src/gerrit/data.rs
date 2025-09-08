use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Deserialize, Clone, Debug)]
pub struct ApprovalInfo {
    pub username: String,
    #[serde(default)]
    pub value: i64,
}

#[derive(Deserialize, Debug)]
pub struct LabelInfoRaw {
    #[serde(default)]
    pub all: Vec<ApprovalInfo>,
}

#[derive(Debug, Clone)]
pub struct LabelInfo(pub Vec<ApprovalInfo>);

impl Deref for LabelInfo {
    type Target = Vec<ApprovalInfo>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<LabelInfoRaw> for LabelInfo {
    fn from(raw: LabelInfoRaw) -> Self {
        LabelInfo(raw.all.clone())
    }
}

#[derive(Deserialize, Debug)]
pub struct ChangeInfoRaw {
    pub id: String,
    pub change_id: String,
    pub _number: u64,
    pub project: String,
    pub branch: String,

    pub subject: String,

    pub created: String,
    pub updated: String,

    pub status: String,
    #[serde(default)]
    pub work_in_progress: bool,
    #[serde(default)]
    pub unresolved_comment_count: u64,

    #[serde(default)]
    pub labels: HashMap<String, LabelInfoRaw>,
}

#[derive(Debug, Clone)]
pub struct ChangeInfo {
    pub id: String,
    pub id_number: u64,
    pub change_id: String,
    pub project: String,
    pub branch: String,

    pub subject: String,

    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,

    pub status: String,
    pub work_in_progress: bool,
    pub unresolved_comment_count: u64,

    pub labels: HashMap<String, LabelInfo>,
}

impl From<ChangeInfoRaw> for ChangeInfo {
    fn from(raw: ChangeInfoRaw) -> Self {
        ChangeInfo {
            id: raw.id,
            id_number: raw._number,
            change_id: raw.change_id,
            project: raw.project,
            branch: raw.branch,
            subject: raw.subject,
            created: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::parse_from_str(
                    &raw.created,
                    "%Y-%m-%d %H:%M:%S%.f",
                )
                .unwrap(),
                Utc,
            ),
            updated: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::parse_from_str(
                    &raw.updated,
                    "%Y-%m-%d %H:%M:%S%.f",
                )
                .unwrap(),
                Utc,
            ),
            status: raw.status,
            work_in_progress: raw.work_in_progress,
            unresolved_comment_count: raw.unresolved_comment_count,
            labels: raw
                .labels
                .into_iter()
                .map(|(key, value)| (key, Into::into(value)))
                .collect(),
        }
    }
}
