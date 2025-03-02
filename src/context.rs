use crate::gerrit::connection::SharedConnection as Gerrit;

#[derive(Debug, Clone)]
pub struct ServiceContext {
    pub gerrit: Gerrit,
}

impl ServiceContext {
    pub fn new() -> ServiceContext {
        ServiceContext {
            gerrit: crate::gerrit::connection::new(),
        }
    }
}
