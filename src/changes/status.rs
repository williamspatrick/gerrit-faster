use crate::gerrit::data::ChangeInfo as GerritChange;

#[derive(Debug, Clone)]
pub enum ReviewState {
    Unknown,
    PendingCI,
    FailingCI,
}

fn pending_ci(change: &GerritChange) -> bool {
    for score in change.labels["Verified"].iter() {
        if score.username != "jenkins-openbmc-ci" {
            continue;
        }
        return score.value == 0;
    }
    false
}

fn failing_ci(change: &GerritChange) -> bool {
    for score in change.labels["Verified"].iter() {
        if score.username != "jenkins-openbmc-ci" {
            continue;
        }
        return score.value < 0;
    }
    false
}

pub fn review_state(change: &GerritChange) -> ReviewState {
    if pending_ci(change) {
        return ReviewState::PendingCI;
    }

    if failing_ci(change) {
        return ReviewState::FailingCI;
    }

    ReviewState::Unknown
}
