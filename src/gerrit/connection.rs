use crate::gerrit::data as gerrit_data;
use std::fmt;
use std::sync::{Arc, Mutex};

#[async_trait::async_trait]
pub trait GerritConnection {
    fn get_username(&self) -> String;
    fn get_password(&self) -> String;
    async fn all_open_changes(&self) -> Result<Vec<gerrit_data::ChangeInfo>, reqwest::Error>;
}

pub struct Connection {
    username: String,
    password: String,
}

#[derive(Clone, Debug)]
pub struct SharedConnection {
    connection: Arc<Mutex<Connection>>,
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
impl GerritConnection for SharedConnection {
    fn get_username(&self) -> String {
        self.connection.lock().unwrap().username.clone()
    }

    fn get_password(&self) -> String {
        self.connection.lock().unwrap().password.clone()
    }

    async fn all_open_changes(&self) -> Result<Vec<gerrit_data::ChangeInfo>, reqwest::Error> {
        let result = reqwest::Client::new()
            .get("https://gerrit.openbmc.org/a/changes/?q=status:open+-is:wip&no-limit")
            .basic_auth(self.get_username(), Some(self.get_password()))
            .send()
            .await?;

        let text = result.text().await?;
        // Gerrit requires pruning off the first 4 characters to avoid the
        // magic: )]}'
        let pruned = &text[4..];

        Ok(serde_json::from_str::<Vec<gerrit_data::ChangeInfo>>(&pruned).expect("JSON failed"))
    }
}

pub fn new() -> SharedConnection {
    return SharedConnection {
        connection: Arc::new(Mutex::new(
            Connection {
                username: std::env::var("GERRIT_USERNAME").expect("GERRIT_USERNAME must be set"),
                password: std::env::var("GERRIT_PASSWORD").expect("GERRIT_PASSWORD must be set"),
            }
            .clone(),
        )),
    };
}
