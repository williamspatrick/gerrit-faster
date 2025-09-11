use crate::changes::status as Status;
use crate::gerrit::data::ChangeInfo as GerritChange;
use crate::gerrit::data::ChangeStatus as GerritChangeStatus;

use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Change {
    pub change: GerritChange,
    pub review_state: Status::ReviewState,
}

#[derive(Debug, Clone)]
pub struct Container {
    pub changes: HashMap<u64, Change>,
    pub changes_by_id: HashMap<String, u64>,
}

impl Container {
    pub fn new() -> Container {
        Container {
            changes: HashMap::<u64, Change>::new(),
            changes_by_id: HashMap::<String, u64>::new(),
        }
    }

    pub fn set(&mut self, change: &GerritChange) {
        info!("Change: {:?}", change);
        if change.status != GerritChangeStatus::New {
            info!("Dropping due to status={:?}", change.status);
            self.changes.remove(&change.id_number);
            self.changes_by_id.remove(&change.change_id);
        } else {
            let review_state = Status::review_state(change);
            info!("Change Status = {:?}", review_state);
            self.changes.insert(
                change.id_number,
                Change {
                    change: change.clone(),
                    review_state: review_state,
                },
            );
            self.changes_by_id
                .insert(change.change_id.clone(), change.id_number);
        }
    }

    pub fn get(&self, id: u64) -> Option<Change> {
        self.changes.get(&id).cloned()
    }

    pub fn get_by_change_id(&self, id: &String) -> Option<Change> {
        self.get(*self.changes_by_id.get(id)?)
    }
}
