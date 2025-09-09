use crate::context::ServiceContext;
use crate::gerrit::connection::GerritConnection;
use crate::gerrit::data::{ApprovalInfo, ChangeInfo};
use chrono::Utc;
use tracing::info;

async fn abandon_older_than_two_years(
    context: &ServiceContext,
    change: &ChangeInfo,
) -> bool {
    let now = Utc::now();
    let two_years_ago = now - chrono::Duration::days(2 * 365);

    // Check if the change is older than two years
    if change.updated >= two_years_ago {
        return false;
    }

    info!(
        "Abandoning change {} (last updated: {})",
        change.id, change.updated
    );

    // Call the abandon_change method
    match context
            .gerrit
            .abandon_change(
                &change.id,
                concat!(
                    "Automatically abandoned due to inactivity of over two years.\n",
                    "Please rebase and reopen if this should still be merged.",
                ).to_string(),
            )
            .await
    {
        Ok(_) => { info!("Successfully abandoned change {}", change.id); return true; },
        Err(e) => info!("Failed to abandon change {}: {}", change.id, e),
    }

    false
}

async fn abandon_older_than_one_year_and_bad_ci(
    context: &ServiceContext,
    change: &ChangeInfo,
) -> bool {
    let now = Utc::now();
    let one_year_ago = now - chrono::Duration::days(1 * 365);

    // Check if the change is older than one year
    if change.updated >= one_year_ago {
        return false;
    }

    let mut found_verified: Option<ApprovalInfo> = None;
    for score in change.labels["Verified"].iter() {
        if score.username != "jenkins-openbmc-ci" || score.value > 0 {
            continue;
        }
        found_verified = Some(score.clone());
    }

    if !found_verified.is_some() {
        return false;
    }
    let ci_failure = found_verified.unwrap();

    info!(
        "Abandoning change {} (not verified: {}, {})",
        change.id, ci_failure.username, ci_failure.value,
    );

    // Call the abandon_change method
    match context
                .gerrit
                .abandon_change(
                    &change.id,
                    concat!(
                        "Automatically abandoned due to missing or failing CI and inactivity of over one year.\n",
                        "Please rebase, resolve CI and reopen if this should still be merged.\n",
                        "\n",
                    )
                    .to_string()
                    + &format!(
                        "CI status: {}={}",
                        ci_failure.username,
                        ci_failure.value
                    )
                )
                .await
            {
                Ok(_) => { info!("Successfully abandoned change {}", change.id); return true},
                Err(e) => info!("Failed to abandon change {}: {}", change.id, e),
            }

    false
}

pub async fn serve(context: ServiceContext) {
    let changes = context.gerrit.all_open_changes().await.unwrap();
    let mut abandoned = 0;

    let mut all_changes = context.changes.lock().unwrap();

    for change in &changes {
        all_changes.set(change);

        if abandon_older_than_two_years(&context, change).await
            || abandon_older_than_one_year_and_bad_ci(&context, change).await
        {
            abandoned += 1;
        }
    }

    info!("Total Changes: {:?}", changes.len());
    info!("Total abandoned: {:?}", abandoned);
}
