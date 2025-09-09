use crate::changes::container::Container as Changes;
use crate::gerrit::connection::SharedConnection as Gerrit;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ServiceContext {
    pub gerrit: Gerrit,
    pub changes: Arc<Mutex<Changes>>,
}

impl ServiceContext {
    pub fn new() -> ServiceContext {
        ServiceContext {
            gerrit: crate::gerrit::connection::new(),
            changes: Arc::<Mutex<Changes>>::new(Mutex::new(Changes::new())),
        }
    }
}
