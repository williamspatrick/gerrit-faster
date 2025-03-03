use std::fmt;
use std::sync::{Arc, Mutex};

pub trait GerritConnection {
    fn get_username(&self) -> String;
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

impl GerritConnection for SharedConnection {
    fn get_username(&self) -> String {
        self.connection.lock().unwrap().username.clone()
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
