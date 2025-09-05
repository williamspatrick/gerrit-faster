use crate::context::ServiceContext;
use crate::gerrit::connection::GerritConnection;
use tracing::info;

pub async fn serve(context: ServiceContext) {
    let changes = context.gerrit.all_open_changes().await.unwrap();

    for change in &changes {
        info!("Change: {:?}", change);
    }

    info!("Total Changes: {:?}", changes.len());
}
