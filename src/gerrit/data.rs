use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Deserialize, Clone, Debug)]
pub struct AccountInfo {
    pub username: String,
}

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

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubmitRecordStatus {
    Ok,
    NotReady,
    Closed,
    Forced,
    RuleError,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SubmitRecord {
    pub rule_name: String,
    pub status: SubmitRecordStatus,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeStatus {
    New,
    Merged,
    Abandoned,
}

#[derive(Deserialize, Debug)]
pub struct ChangeInfoRaw {
    pub id: String,
    pub change_id: String,
    pub _number: u64,
    pub project: String,
    pub branch: String,

    pub subject: String,
    pub owner: AccountInfo,

    pub created: String,
    pub updated: String,

    pub status: ChangeStatus,
    #[serde(default)]
    pub work_in_progress: bool,
    #[serde(default)]
    pub mergeable: bool,
    #[serde(default)]
    pub unresolved_comment_count: u64,

    #[serde(default)]
    pub labels: HashMap<String, LabelInfoRaw>,
    #[serde(default)]
    pub submit_records: Vec<SubmitRecord>,
}

#[derive(Debug, Clone)]
pub struct ChangeInfo {
    pub id: String,
    pub id_number: u64,
    pub change_id: String,
    pub project: String,
    pub branch: String,

    pub subject: String,
    pub owner: AccountInfo,

    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,

    pub status: ChangeStatus,
    pub work_in_progress: bool,
    pub mergeable: bool,
    pub unresolved_comment_count: u64,

    pub labels: HashMap<String, LabelInfo>,
    pub submit_records: Vec<SubmitRecord>,
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
            owner: raw.owner,
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
            mergeable: raw.mergeable,
            unresolved_comment_count: raw.unresolved_comment_count,
            labels: raw
                .labels
                .into_iter()
                .map(|(key, value)| (key, Into::into(value)))
                .collect(),
            submit_records: raw.submit_records.clone(),
        }
    }
}
