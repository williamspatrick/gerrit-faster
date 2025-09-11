use crate::gerrit::data as gerrit_data;
use serde_json;
use std::fmt;

/* Gerrit JSON responses have a magic at the beginning that needs to be
 * stripped. */
fn prune_magic(text: String) -> String {
    const MAGIC_PREFIX: &str = ")]}'";
    if text.starts_with(MAGIC_PREFIX) {
        return text[MAGIC_PREFIX.len()..].to_string();
    }
    text
}

#[async_trait::async_trait]
pub trait GerritConnection {
    fn get_username(&self) -> String;
    fn get_password(&self) -> String;
    async fn execute_request(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<String, reqwest::Error>;
    async fn all_open_changes(
        &self,
    ) -> Result<Vec<gerrit_data::ChangeInfo>, reqwest::Error>;
    async fn recent_changes(
        &self,
    ) -> Result<Vec<gerrit_data::ChangeInfo>, reqwest::Error>;
    async fn abandon_change(
        &self,
        change_id: &str,
        message: String,
    ) -> Result<gerrit_data::ChangeInfo, reqwest::Error>;
}

pub struct Connection {
    username: String,
    password: String,
}

impl Clone for Connection {
    fn clone(&self) -> Connection {
        Connection {
            username: self.username.clone(),
            password: self.password.clone(),
        }
    }
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Connection")
            .field("username", &self.username)
            .field("password", &"xxxxxxxx")
            .finish()
    }
}

#[async_trait::async_trait]
impl GerritConnection for Connection {
    fn get_username(&self) -> String {
        self.username.clone()
    }

    fn get_password(&self) -> String {
        self.password.clone()
    }

    async fn execute_request(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<String, reqwest::Error> {
        Ok(prune_magic(
            request
                .basic_auth(self.get_username(), Some(self.get_password()))
                .send()
                .await?
                .text()
                .await?,
        ))
    }

    async fn all_open_changes(
        &self,
    ) -> Result<Vec<gerrit_data::ChangeInfo>, reqwest::Error> {
        let result = self.execute_request(reqwest::Client::new()
            .get("https://gerrit.openbmc.org/a/changes/?q=status:open+-is:wip&o=LABELS&o=DETAILED_ACCOUNTS&no-limit")).await?;

        Ok(
            serde_json::from_str::<Vec<gerrit_data::ChangeInfoRaw>>(&result)
                .expect("JSON failed")
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }

    async fn recent_changes(
        &self,
    ) -> Result<Vec<gerrit_data::ChangeInfo>, reqwest::Error> {
        let result = self.execute_request(reqwest::Client::new()
            .get("https://gerrit.openbmc.org/a/changes/?q=status:open+-is:wip+-age:1h&o=LABELS&o=DETAILED_ACCOUNTS&no-limit")).await?;

        Ok(
            serde_json::from_str::<Vec<gerrit_data::ChangeInfoRaw>>(&result)
                .expect("JSON failed")
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }

    async fn abandon_change(
        &self,
        change_id: &str,
        message: String,
    ) -> Result<gerrit_data::ChangeInfo, reqwest::Error> {
        let url = format!(
            "https://gerrit.openbmc.org/a/changes/{}/abandon",
            change_id
        );

        let mut request_body = serde_json::Map::new();
        request_body
            .insert("message".to_string(), serde_json::Value::String(message));

        let result = self
            .execute_request(
                reqwest::Client::new()
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .json(&request_body),
            )
            .await?;

        Ok(serde_json::from_str::<gerrit_data::ChangeInfoRaw>(&result)
            .expect("JSON failed")
            .into())
    }
}

pub fn new() -> Connection {
    Connection {
        username: std::env::var("GERRIT_USERNAME")
            .expect("GERRIT_USERNAME must be set"),
        password: std::env::var("GERRIT_PASSWORD")
            .expect("GERRIT_PASSWORD must be set"),
    }
}
