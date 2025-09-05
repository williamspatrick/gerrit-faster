use crate::context::ServiceContext;
use crate::gerrit::connection::GerritConnection;
use chrono::Utc;
use tracing::info;

pub async fn serve(context: ServiceContext) {
    let changes = context.gerrit.all_open_changes().await.unwrap();
    let now = Utc::now();
    let two_years_ago = now - chrono::Duration::days(2 * 365);

    let mut abandoned = 0;
    for change in &changes {
        info!("Change: {:?}", change);

        // Check if the change is older than a year
        if change.updated < two_years_ago {
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
                Ok(_) => { info!("Successfully abandoned change {}", change.id); abandoned += 1; },
                Err(e) => info!("Failed to abandon change {}: {}", change.id, e),
            }
        }
    }

    info!("Total Changes: {:?}", changes.len());
    info!("Total abandoned: {:?}", abandoned);
}
