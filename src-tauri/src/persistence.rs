use std::{error::Error, fmt};

use sqlx::SqlitePool;
use tauri::{Manager, Runtime};
use tauri_plugin_sql::{DbInstances, DbPool};

use crate::DATABASE_URL;

/// The plugin-managed SQLite pool that later persistence phases will use.
#[derive(Clone)]
pub(crate) struct SharedSqlitePool {
    pool: SqlitePool,
}

impl SharedSqlitePool {
    fn from_manager<R: Runtime>(manager: &impl Manager<R>) -> Result<Self, SharedPoolError> {
        let instances = manager
            .try_state::<DbInstances>()
            .ok_or(SharedPoolError::PluginStateUnavailable)?;

        Self::from_instances(&instances)
    }

    fn from_instances(instances: &DbInstances) -> Result<Self, SharedPoolError> {
        let instances = instances
            .0
            .try_read()
            .map_err(|_| SharedPoolError::DatabaseMapUnavailable)?;
        let database = instances
            .get(DATABASE_URL)
            .ok_or(SharedPoolError::DatabaseNotPreloaded(DATABASE_URL))?;

        let DbPool::Sqlite(pool) = database;

        Ok(Self { pool: pool.clone() })
    }

    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

/// Installs a narrow Rust owner for the exact pool opened and migrated by the SQL plugin.
///
/// Tauri initializes plugins during `Builder::build`, before this application setup hook
/// runs. The SQL plugin manages `DbInstances` only after every configured preload has
/// connected and its registered migrations have completed, so successful resolution here
/// is the repository-readiness barrier.
pub(crate) fn install_shared_sqlite_pool<R: Runtime>(
    manager: &impl Manager<R>,
) -> Result<(), SharedPoolError> {
    let database = SharedSqlitePool::from_manager(manager)?;

    // This annotation compile-proves compatibility with the direct SQLx 0.8.6 type.
    let _: &SqlitePool = database.pool();

    if !manager.manage(database) {
        return Err(SharedPoolError::OwnerAlreadyManaged);
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SharedPoolError {
    PluginStateUnavailable,
    DatabaseMapUnavailable,
    DatabaseNotPreloaded(&'static str),
    OwnerAlreadyManaged,
}

impl fmt::Display for SharedPoolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PluginStateUnavailable => {
                write!(formatter, "the SQL plugin database state is unavailable")
            }
            Self::DatabaseMapUnavailable => {
                write!(formatter, "the SQL plugin database map is unavailable")
            }
            Self::DatabaseNotPreloaded(database_url) => {
                write!(
                    formatter,
                    "the configured database was not preloaded: {database_url}"
                )
            }
            Self::OwnerAlreadyManaged => {
                write!(formatter, "the shared SQLite pool owner is already managed")
            }
        }
    }
}

impl Error for SharedPoolError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn lazy_pool() -> SqlitePool {
        SqlitePool::connect_lazy("sqlite::memory:")
            .expect("the SQLx SQLite URL fixture should be valid")
    }

    #[test]
    fn resolves_the_exact_preloaded_database_as_the_direct_sqlx_pool_type() {
        tauri::async_runtime::block_on(async {
            let instances = DbInstances::default();
            {
                let mut databases = instances
                    .0
                    .try_write()
                    .expect("the database fixture should not be locked");
                databases.insert("sqlite:other.db".to_owned(), DbPool::Sqlite(lazy_pool()));
                databases.insert(DATABASE_URL.to_owned(), DbPool::Sqlite(lazy_pool()));
            }

            let database = SharedSqlitePool::from_instances(&instances)
                .expect("the exact preloaded database key should resolve");
            let pool: &SqlitePool = database.pool();

            assert!(!pool.is_closed());
        });
    }

    #[test]
    fn does_not_fall_back_to_another_database_entry() {
        tauri::async_runtime::block_on(async {
            let instances = DbInstances::default();
            instances
                .0
                .try_write()
                .expect("the database fixture should not be locked")
                .insert("sqlite:other.db".to_owned(), DbPool::Sqlite(lazy_pool()));

            assert!(matches!(
                SharedSqlitePool::from_instances(&instances),
                Err(SharedPoolError::DatabaseNotPreloaded(DATABASE_URL))
            ));
        });
    }
}
