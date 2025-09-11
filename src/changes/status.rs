use crate::gerrit::data as GerritData;

#[derive(Clone)]
pub enum ReviewState {
    Unknown,
    MissingCI,
    FailingCI,
    PendingFeedback(String),
    PendingCommentResolution(u64),
    CommunityReview,
    MaintainerReview,
    ReadyToSubmit,
}

impl std::fmt::Debug for ReviewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewState::Unknown => write!(f, "Unknown"),
            ReviewState::MissingCI => write!(f, "Missing CI"),
            ReviewState::FailingCI => write!(f, "Failing CI"),
            ReviewState::PendingFeedback(user) => {
                write!(f, "Pending Feedback (by {})", user)
            }
            ReviewState::PendingCommentResolution(count) => {
                write!(f, "Pending Comment Resolution ({} pending)", count)
            }
            ReviewState::CommunityReview => write!(f, "Community Review"),
            ReviewState::MaintainerReview => write!(f, "Maintainer Review"),
            ReviewState::ReadyToSubmit => write!(f, "Ready to Submit"),
        }
    }
}

fn pending_ci(change: &GerritData::ChangeInfo) -> ReviewState {
    for score in change.labels["Verified"].iter() {
        if score.username != "jenkins-openbmc-ci" {
            continue;
        }
        if score.value == 0 {
            return ReviewState::MissingCI;
        } else {
            return ReviewState::Unknown;
        }
    }
    ReviewState::MissingCI
}

fn failing_ci(change: &GerritData::ChangeInfo) -> ReviewState {
    for score in change.labels["Verified"].iter() {
        if score.username != "jenkins-openbmc-ci" {
            continue;
        }
        if score.value < 0 {
            return ReviewState::FailingCI;
        } else {
            return ReviewState::Unknown;
        }
    }
    ReviewState::Unknown
}

fn pending_feedback(change: &GerritData::ChangeInfo) -> ReviewState {
    for score in change.labels["Code-Review"].iter() {
        if score.username == change.owner.username {
            continue;
        }
        if score.value < 0 {
            return ReviewState::PendingFeedback(score.username.clone());
        } else {
            return ReviewState::Unknown;
        }
    }
    ReviewState::Unknown
}

fn pending_comments(change: &GerritData::ChangeInfo) -> ReviewState {
    if change.unresolved_comment_count != 0 {
        return ReviewState::PendingCommentResolution(
            change.unresolved_comment_count,
        );
    }
    ReviewState::Unknown
}

fn no_reviews(change: &GerritData::ChangeInfo) -> ReviewState {
    for score in change.labels["Code-Review"].iter() {
        if score.username == change.owner.username {
            continue;
        }
        if score.value != 0 {
            return ReviewState::Unknown;
        }
    }
    ReviewState::CommunityReview
}

fn missing_maintainer_review(change: &GerritData::ChangeInfo) -> ReviewState {
    for submit_record in change.submit_records.iter() {
        if submit_record.rule_name != "owners~OwnersSubmitRequirement" {
            continue;
        }
        match submit_record.status {
            GerritData::SubmitRecordStatus::NotReady => {
                return ReviewState::MaintainerReview
            }
            GerritData::SubmitRecordStatus::Ok => {
                return ReviewState::ReadyToSubmit
            }
            _ => {}
        }
    }
    ReviewState::Unknown
}

pub fn review_state(change: &GerritData::ChangeInfo) -> ReviewState {
    let mut status;

    status = pending_ci(change);
    if !matches!(status, ReviewState::Unknown) {
        return status;
    }

    status = failing_ci(change);
    if !matches!(status, ReviewState::Unknown) {
        return status;
    }

    status = pending_feedback(change);
    if !matches!(status, ReviewState::Unknown) {
        return status;
    }

    status = pending_comments(change);
    if !matches!(status, ReviewState::Unknown) {
        return status;
    }

    status = no_reviews(change);
    if !matches!(status, ReviewState::Unknown) {
        return status;
    }

    status = missing_maintainer_review(change);
    if !matches!(status, ReviewState::Unknown) {
        return status;
    }

    ReviewState::Unknown
}
