use tauri_plugin_sql::{Migration, MigrationKind};

pub mod domain;
mod persistence;

pub(crate) const DATABASE_URL: &str = "sqlite:nous.db";

fn migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "initialize_local_storage",
        sql: include_str!("../migrations/0001_initialize.sql"),
        kind: MigrationKind::Up,
    }]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(DATABASE_URL, migrations())
                .build(),
        )
        .setup(|app| {
            persistence::install_shared_sqlite_pool(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running NOUS");
}

#[cfg(test)]
mod tests {
    use super::migrations;

    #[test]
    fn contains_only_the_initial_infrastructure_migration() {
        let migrations = migrations();

        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[0].description, "initialize_local_storage");
        assert!(migrations[0].sql.contains("app_metadata"));
    }
}
