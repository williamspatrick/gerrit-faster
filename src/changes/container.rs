use crate::changes::status as Status;
use crate::gerrit::data::ChangeInfo as GerritChange;
use crate::gerrit::data::ChangeStatus as GerritChangeStatus;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct Change {
    pub change: GerritChange,
    pub review_state: Status::ReviewState,
    pub review_state_updated: DateTime<Utc>,
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
        debug!("Change: {:?}", change);
        if change.status != GerritChangeStatus::New {
            if self.changes.contains_key(&change.id_number) {
                debug!("Dropping due to status={:?}", change.status);
                self.remove(change);
            }
        } else if change.work_in_progress {
            if self.changes.contains_key(&change.id_number) {
                debug!("Dropping due to WIP");
                self.remove(change);
            }
        } else {
            let review_state = Status::review_state(change);
            debug!("Change Status = {:?}", review_state);

            let review_state_updated =
                if let Some(i) = self.changes.get(&change.id_number) {
                    if i.review_state == review_state {
                        i.review_state_updated
                    } else {
                        change.updated
                    }
                } else {
                    change.updated
                };

            self.changes.insert(
                change.id_number,
                Change {
                    change: change.clone(),
                    review_state: review_state,
                    review_state_updated: review_state_updated,
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

    pub fn remove(&mut self, change: &GerritChange) {
        self.changes.remove(&change.id_number);
        self.changes_by_id.remove(&change.change_id);
    }
}
