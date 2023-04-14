use log;
use sqlx;
use sqlx::pool::PoolConnection;
use sqlx::prelude::*;
use sqlx::sqlite::SqlitePool;
use sqlx::Sqlite;

const MAX_PATH_SIZE: usize = 2048;
const DB_PATH: &str = "database.sqlite";

#[derive(Clone)]
pub struct Database {
    connection: SqlitePool,
}

pub struct AcquiredConnection {
    connection: PoolConnection<Sqlite>,
}

pub struct FolderWrapper {
    pub id: i64,
    pub path: String,
}

impl Database {
    pub async fn connect(connection_path: String) -> Self {
        let connection = SqlitePool::connect(&connection_path)
            .await
            .expect("Can not open sqlite db");

        Database { connection }
    }

    pub async fn connect_default() -> Self {
        Self::connect(DB_PATH.to_string()).await
    }

    pub async fn migrate(&self) {
        self.connection
            .execute(
                " 
                CREATE TABLE IF NOT EXISTS folders (
                    id INTEGER PRIMARY KEY,
                    path TEXT(2048) UNIQUE
                )
               ",
            )
            .await
            .expect("Can not create folders table");

        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS images (
                    id INTEGER PRIMARY KEY,
                    path TEXT(2048) UNIQUE,
                    hash TEXT(256)
                )
                ",
            )
            .await
            .expect("Can not create images table");
    }

    pub async fn get_connection(&self) -> AcquiredConnection {
        let connection = self.connection.acquire().await.unwrap();

        AcquiredConnection { connection }
    }
}

impl AcquiredConnection {
    // some folders may not be inserted
    pub async fn insert_folders(&mut self, paths: Vec<String>) -> Vec<FolderWrapper> {
        let mut result = Vec::new();

        for path in paths {
            if path.len() > MAX_PATH_SIZE {
                log::warn!("{} path is too long and can not be inserted into DB", path);
                continue;
            }

            println!("INSERTING");
            let record_id = sqlx::query("INSERT INTO folders(path) VALUES (?)")
                .bind(&path)
                .execute(&mut self.connection)
                .await;

            if let Ok(record_id) = record_id {
                result.push(FolderWrapper {
                    id: record_id.last_insert_rowid(),
                    path: path.clone(),
                });
            }
        }

        result
    }
}
