use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct ApprovalInfoRaw {
    pub username: String,
    pub value: Option<i64>,
}

#[derive(Debug)]
pub struct ApprovalInfo {
    pub username: String,
    pub value: i64,
}

impl From<ApprovalInfoRaw> for ApprovalInfo {
    fn from(raw: ApprovalInfoRaw) -> Self {
        ApprovalInfo {
            username: raw.username,
            value: raw.value.unwrap_or(0),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct LabelInfoRaw {
    pub all: Option<Vec<ApprovalInfoRaw>>,
}

#[derive(Debug)]
pub struct LabelInfo(pub Vec<ApprovalInfo>);

impl From<LabelInfoRaw> for LabelInfo {
    fn from(raw: LabelInfoRaw) -> Self {
        LabelInfo(
            raw.all
                .unwrap_or(Vec::<ApprovalInfoRaw>::new())
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct ChangeInfoRaw {
    pub id: String,
    pub change_id: String,
    pub project: String,
    pub branch: String,

    pub subject: String,

    pub created: String,
    pub updated: String,

    pub status: String,
    pub work_in_progress: Option<bool>,

    pub labels: Option<HashMap<String, LabelInfoRaw>>,
}

#[derive(Debug)]
pub struct ChangeInfo {
    pub id: String,
    pub change_id: String,
    pub project: String,
    pub branch: String,

    pub subject: String,

    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,

    pub status: String,
    pub work_in_progress: bool,

    pub labels: HashMap<String, LabelInfo>,
}

impl From<ChangeInfoRaw> for ChangeInfo {
    fn from(raw: ChangeInfoRaw) -> Self {
        ChangeInfo {
            id: raw.id,
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
            work_in_progress: raw.work_in_progress.unwrap_or(false),
            labels: raw
                .labels
                .unwrap_or(HashMap::<String, LabelInfoRaw>::new())
                .into_iter()
                .map(|(key, value)| (key, Into::into(value)))
                .collect(),
        }
    }
}
