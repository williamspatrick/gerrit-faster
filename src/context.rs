use crate::changes::container::Container as Changes;
use crate::gerrit::connection::SharedConnection as Gerrit;

#[derive(Debug, Clone)]
pub struct ServiceContext {
    pub gerrit: Gerrit,
    pub changes: Changes,
}

impl ServiceContext {
    pub fn new() -> ServiceContext {
        ServiceContext {
            gerrit: crate::gerrit::connection::new(),
            changes: Changes::new(),
        }
    }
}
