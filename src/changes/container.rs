use crate::gerrit::data::ChangeInfo;
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Container {
    pub changes: HashMap<String, ChangeInfo>,
}

impl Container {
    pub fn new() -> Container {
        Container {
            changes: HashMap::<String, ChangeInfo>::new(),
        }
    }

    pub fn set(&mut self, change: &ChangeInfo) {
        info!("Change: {:?}", change);
        if change.status != "NEW" {
            info!("Dropping due to status={}", change.status);
            self.changes.remove(&change.id);
        } else {
            self.changes.insert(change.id.clone(), change.clone());
        }
    }
}
