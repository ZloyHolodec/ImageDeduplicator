use crate::database::Database;
use crate::database::FolderWrapper;
use tokio;

#[tokio::main]
pub async fn insert_new_folders(paths: Vec<String>) -> Vec<FolderWrapper> {
    let connection = Database::connect_default().await;
    let mut connection_pool = connection.get_connection().await;
    connection_pool.insert_folders(paths).await
}
