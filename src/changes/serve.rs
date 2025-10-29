use crate::context::ServiceContext;
use crate::gerrit::connection::GerritConnection;
use crate::gerrit::data::{ApprovalInfo, ChangeInfo};
use chrono::{TimeZone, Utc};
use tokio::time::{Duration, sleep};
use tracing::{debug, info, warn};

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
    if let Some(_) = context
        .get_gerrit()
        .abandon_change(
            &change.id,
            concat!(
                "Automatically abandoned due to inactivity of over two years.\n",
                "Please rebase and reopen if this should still be merged.",
            ).to_string(),
        )
        .await
    {
        info!("Successfully abandoned change {}", change.id);
        return true;
    }

    warn!("Was not able to abandon change {}.", change.id);
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
    if let Some(_) = context
        .get_gerrit()
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
        info!("Successfully abandoned change {}", change.id);
        return true;
    }
    warn!("Was not able to abandon change {}", change.id);
    false
}

pub async fn serve(context: ServiceContext) {
    let mut last_full_sync = Utc.timestamp_opt(0, 0).unwrap();

    loop {
        let changes =
            if Utc::now().signed_duration_since(last_full_sync).num_days() >= 1
            {
                last_full_sync = Utc::now();
                debug!("Performing daily full sync of open changes");
                context.get_gerrit().all_open_changes().await
            } else {
                context.get_gerrit().recent_changes().await
            };

        for change in &changes {
            context.lock().unwrap().changes.set(change);

            if abandon_older_than_two_years(&context, change).await
                || abandon_older_than_one_year_and_bad_ci(&context, change)
                    .await
            {
                context.lock().unwrap().changes.remove(change);
            }
        }

        sleep(Duration::from_secs(60)).await;
    }
}
