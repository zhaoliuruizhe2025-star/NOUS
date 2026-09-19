use serde::Serialize;
use tauri::State;

use crate::persistence::SharedSqlitePool;

const EXPECTED_SCHEMA_VERSION: &str = "2";
const READINESS_ERROR: &str = "Local database readiness verification failed.";

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatabaseStatus {
    schema_version: u32,
}

/// Reports readiness for the one database managed and migrated by the SQL plugin.
///
/// The command deliberately accepts no caller-selected SQL, table, path, or operation.
#[tauri::command]
pub(crate) async fn database_status(
    database: State<'_, SharedSqlitePool>,
) -> Result<DatabaseStatus, &'static str> {
    database_status_for_pool(&database).await
}

async fn database_status_for_pool(
    database: &SharedSqlitePool,
) -> Result<DatabaseStatus, &'static str> {
    let mut connection = database
        .acquire_verified_connection()
        .await
        .map_err(|_| READINESS_ERROR)?;
    let schema_version = sqlx::query_scalar::<_, String>(
        "SELECT value FROM app_metadata WHERE key = 'schema_version' LIMIT 1",
    )
    .fetch_optional(&mut **connection.connection())
    .await
    .map_err(|_| READINESS_ERROR)?;

    status_for_schema_version(schema_version.as_deref())
}

fn status_for_schema_version(schema_version: Option<&str>) -> Result<DatabaseStatus, &'static str> {
    if schema_version != Some(EXPECTED_SCHEMA_VERSION) {
        return Err(READINESS_ERROR);
    }

    Ok(DatabaseStatus { schema_version: 2 })
}

#[cfg(test)]
mod tests {
    use super::{
        database_status_for_pool, status_for_schema_version, DatabaseStatus, READINESS_ERROR,
    };
    use crate::persistence::SharedSqlitePool;
    use sqlx::sqlite::SqlitePoolOptions;

    #[test]
    fn reports_ready_only_for_the_expected_schema_version() {
        assert_eq!(
            status_for_schema_version(Some("2")),
            Ok(DatabaseStatus { schema_version: 2 })
        );
        assert_eq!(status_for_schema_version(Some("1")), Err(READINESS_ERROR));
        assert_eq!(status_for_schema_version(None), Err(READINESS_ERROR));
    }

    #[test]
    fn serializes_the_small_frontend_status_shape() {
        assert_eq!(
            serde_json::to_value(DatabaseStatus { schema_version: 2 })
                .expect("database status should serialize"),
            serde_json::json!({ "schemaVersion": 2 })
        );
    }

    #[test]
    fn database_failures_and_unready_connections_are_sanitized() {
        tauri::async_runtime::block_on(async {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await
                .expect("readiness test database should open");
            let database = SharedSqlitePool::from_test_pool(pool.clone());

            assert_eq!(
                database_status_for_pool(&database).await,
                Err(READINESS_ERROR)
            );

            sqlx::query(
                "CREATE TABLE app_metadata (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)",
            )
            .execute(&pool)
            .await
            .unwrap();
            assert_eq!(
                database_status_for_pool(&database).await,
                Err(READINESS_ERROR)
            );

            sqlx::query("INSERT INTO app_metadata (key, value) VALUES ('schema_version', '1')")
                .execute(&pool)
                .await
                .unwrap();
            assert_eq!(
                database_status_for_pool(&database).await,
                Err(READINESS_ERROR)
            );

            sqlx::query("UPDATE app_metadata SET value = '2' WHERE key = 'schema_version'")
                .execute(&pool)
                .await
                .unwrap();
            assert_eq!(
                database_status_for_pool(&database).await,
                Ok(DatabaseStatus { schema_version: 2 })
            );

            let mut connection = pool.acquire().await.unwrap();
            sqlx::query("PRAGMA foreign_keys = OFF")
                .execute(&mut *connection)
                .await
                .unwrap();
            drop(connection);
            assert_eq!(
                database_status_for_pool(&database).await,
                Err(READINESS_ERROR)
            );

            pool.close().await;
        });
    }
}
