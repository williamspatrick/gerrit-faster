use crate::changes::status as Status;
use crate::gerrit::data::ChangeInfo as GerritChange;

use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Change {
    pub change: GerritChange,
    pub review_state: Status::ReviewState,
}

#[derive(Debug, Clone)]
pub struct Container {
    pub changes: HashMap<String, Change>,
}

impl Container {
    pub fn new() -> Container {
        Container {
            changes: HashMap::<String, Change>::new(),
        }
    }

    pub fn set(&mut self, change: &GerritChange) {
        info!("Change: {:?}", change);
        if change.status != "NEW" {
            info!("Dropping due to status={}", change.status);
            self.changes.remove(&change.id);
        } else {
            let review_state = Status::review_state(change);
            info!("Change Status = {:?}", review_state);
            self.changes.insert(
                change.id.clone(),
                Change {
                    change: change.clone(),
                    review_state: review_state,
                },
            );
        }
    }
}
