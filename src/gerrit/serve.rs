use crate::context::ServiceContext;
use crate::gerrit::connection::GerritConnection;
use crate::gerrit::data::ChangeInfo;
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
                Some(
                    "Automatically abandoned due to inactivity of over two years. Please rebase and reopen if this should still be merged.",
                ),
            )
            .await
    {
        Ok(_) => { info!("Successfully abandoned change {}", change.id); return true; },
        Err(e) => info!("Failed to abandon change {}: {}", change.id, e),
    }

    false
}

pub async fn serve(context: ServiceContext) {
    let changes = context.gerrit.all_open_changes().await.unwrap();
    let mut abandoned = 0;

    for change in &changes {
        info!("Change: {:?}", change);

        if abandon_older_than_two_years(&context, change).await {
            abandoned += 1;
        }
    }

    info!("Total Changes: {:?}", changes.len());
    info!("Total abandoned: {:?}", abandoned);
}
