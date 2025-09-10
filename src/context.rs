use crate::changes::container::Container as Changes;
use crate::gerrit::connection::SharedConnection as Gerrit;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ServiceContextData {
    pub gerrit: Gerrit,
    pub changes: Changes,
}

#[derive(Debug, Clone)]
pub struct ServiceContext(pub Arc<Mutex<ServiceContextData>>);

impl ServiceContext {
    pub fn new() -> ServiceContext {
        ServiceContext(Arc::new(Mutex::new(ServiceContextData {
            gerrit: crate::gerrit::connection::new(),
            changes: Changes::new(),
        })))
    }

    pub fn lock(
        &self,
    ) -> Result<
        std::sync::MutexGuard<'_, ServiceContextData>,
        std::sync::PoisonError<std::sync::MutexGuard<'_, ServiceContextData>>,
    > {
        self.0.lock()
    }

    pub fn get_gerrit(&self) -> Gerrit {
        self.lock().unwrap().gerrit.clone()
    }
}
