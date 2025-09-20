use crate::gerrit::data as gerrit_data;
use serde_json;
use std::fmt;
use tokio::time::{self, Duration};
use tracing::{error, info};

/* Gerrit JSON responses have a magic at the beginning that needs to be
 * stripped. */
fn prune_magic(text: String) -> Option<String> {
    const MAGIC_PREFIX: &str = ")]}'";
    if text.starts_with(MAGIC_PREFIX) {
        return Some(text[MAGIC_PREFIX.len()..].to_string());
    }
    None
}

#[async_trait::async_trait]
pub trait GerritConnection {
    fn get_username(&self) -> String;
    fn get_password(&self) -> String;
    async fn execute_request<F>(
        &self,
        request: reqwest::RequestBuilder,
        acceptable_status: F,
    ) -> String
    where
        F: Fn(reqwest::StatusCode) -> bool + Send;
    async fn all_open_changes(&self) -> Vec<gerrit_data::ChangeInfo>;
    async fn recent_changes(&self) -> Vec<gerrit_data::ChangeInfo>;
    async fn abandon_change(
        &self,
        change_id: &str,
        message: String,
    ) -> Option<gerrit_data::ChangeInfo>;
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

    async fn execute_request<F>(
        &self,
        request: reqwest::RequestBuilder,
        acceptable_status: F,
    ) -> String
    where
        F: Fn(reqwest::StatusCode) -> bool + Send,
    {
        // Clone the request builder for retries
        let request_factory = || {
            request
                .try_clone()
                .expect("Failed to clone request builder")
        };

        loop {
            let current_request = request_factory();
            match current_request
                .basic_auth(self.get_username(), Some(self.get_password()))
                .send()
                .await
            {
                Ok(response) => {
                    // Check if this is an exception status that should not be retried
                    if acceptable_status(response.status()) {
                        let status = response.status();
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        info!(
                            "Request failed with exception status {}: {}",
                            status, error_text
                        );
                        // For exception statuses, return empty string
                        return String::new();
                    }

                    let text = response.text().await.unwrap();
                    if let Some(pruned) = prune_magic(text) {
                        return pruned;
                    }
                    // If we don't get the magic prefix, wait 10 seconds and retry
                    time::sleep(Duration::from_secs(10)).await;
                }
                Err(_e) => {
                    // If we get an HTTP error, wait 10 seconds and retry
                    time::sleep(Duration::from_secs(10)).await;
                }
            }
        }
    }

    async fn all_open_changes(&self) -> Vec<gerrit_data::ChangeInfo> {
        let result = self
            .execute_request(
                reqwest::Client::new().get(
                    "https://gerrit.openbmc.org/a/changes/?q=status:open+-is:wip&o=LABELS&o=DETAILED_ACCOUNTS&no-limit",
                ),
                |_| false,
            )
            .await;

        serde_json::from_str::<Vec<gerrit_data::ChangeInfoRaw>>(&result)
            .expect(&format!(
                "Failed to parse JSON response for all_open_changes. Response content: {}",
                result
            ))
            .into_iter()
            .map(Into::into)
            .collect()
    }

    async fn recent_changes(&self) -> Vec<gerrit_data::ChangeInfo> {
        let result = self
            .execute_request(
                reqwest::Client::new().get(
                    "https://gerrit.openbmc.org/a/changes/?q=-age:4h&o=LABELS&o=DETAILED_ACCOUNTS&no-limit",
                ),
                |_| false,
            )
            .await;

        serde_json::from_str::<Vec<gerrit_data::ChangeInfoRaw>>(&result)
            .expect(&format!(
                "Failed to parse JSON response for recent_changes. Response content: {}",
                result
            ))
            .into_iter()
            .map(Into::into)
            .collect()
    }

    async fn abandon_change(
        &self,
        change_id: &str,
        message: String,
    ) -> Option<gerrit_data::ChangeInfo> {
        let url = format!(
            "https://gerrit.openbmc.org/a/changes/{}/abandon",
            change_id
        );

        let mut request_body = serde_json::Map::new();
        request_body
            .insert("message".to_string(), serde_json::Value::String(message));

        // Use execute_request with exception handling for HTTP 409 errors
        let result = self
            .execute_request(
                reqwest::Client::new()
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .json(&request_body),
                |status| status == reqwest::StatusCode::CONFLICT,
            )
            .await;

        // If we got an empty string, it means we hit an exception status (409)
        if result.is_empty() {
            error!("Failed to abandon change {} due to conflict", change_id);
            return None;
        }

        Some(
            serde_json::from_str::<gerrit_data::ChangeInfoRaw>(&result)
                .expect(&format!(
                    "Failed to parse JSON response for abandon_change. Response content: {}",
                    result
                ))
                .into(),
        )
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
