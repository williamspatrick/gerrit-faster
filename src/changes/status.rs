use crate::gerrit::data::ChangeInfo as GerritChange;

#[derive(Clone)]
pub enum ReviewState {
    Unknown,
    PendingCI,
    FailingCI,
    PendingFeedback,
    PendingCommentResolution,
    CommunityReview,
}

impl std::fmt::Debug for ReviewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewState::Unknown => write!(f, "Unknown"),
            ReviewState::PendingCI => write!(f, "Pending CI"),
            ReviewState::FailingCI => write!(f, "Failing CI"),
            ReviewState::PendingFeedback => write!(f, "Pending Feedback"),
            ReviewState::PendingCommentResolution => {
                write!(f, "Pending Comment Resolution")
            }
            ReviewState::CommunityReview => write!(f, "Community Review"),
        }
    }
}

fn pending_ci(change: &GerritChange) -> bool {
    for score in change.labels["Verified"].iter() {
        if score.username != "jenkins-openbmc-ci" {
            continue;
        }
        return score.value == 0;
    }
    true
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

fn pending_feedback(change: &GerritChange) -> bool {
    for score in change.labels["Code-Review"].iter() {
        return score.value < 0;
    }
    false
}

fn pending_comments(change: &GerritChange) -> bool {
    change.unresolved_comment_count != 0
}

fn no_reviews(change: &GerritChange) -> bool {
    for score in change.labels["Code-Review"].iter() {
        if score.value != 0 {
            return false;
        }
    }
    true
}

pub fn review_state(change: &GerritChange) -> ReviewState {
    if pending_ci(change) {
        return ReviewState::PendingCI;
    }

    if failing_ci(change) {
        return ReviewState::FailingCI;
    }

    if pending_feedback(change) {
        return ReviewState::PendingFeedback;
    }

    if pending_comments(change) {
        return ReviewState::PendingCommentResolution;
    }

    if no_reviews(change) {
        return ReviewState::CommunityReview;
    }

    ReviewState::Unknown
}
