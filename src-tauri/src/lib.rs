use tauri_plugin_sql::{Migration, MigrationKind};

mod commands;
pub mod domain;
mod persistence;

pub(crate) const DATABASE_URL: &str = "sqlite:nous.db";

fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "initialize_local_storage",
            sql: include_str!("../migrations/0001_initialize.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_self_model",
            sql: include_str!("../migrations/0002_create_self_model.sql"),
            kind: MigrationKind::Up,
        },
    ]
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
        .invoke_handler(tauri::generate_handler![commands::database_status])
        .run(tauri::generate_context!())
        .expect("error while running NOUS");
}

#[cfg(test)]
mod tests {
    use super::migrations;

    #[test]
    fn contains_the_registered_infrastructure_and_self_model_migrations() {
        let migrations = migrations();

        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[0].description, "initialize_local_storage");
        assert!(migrations[0].sql.contains("app_metadata"));
        assert_eq!(migrations[1].version, 2);
        assert_eq!(migrations[1].description, "create_self_model");
        assert!(migrations[1].sql.contains("CREATE TABLE \"values\""));
    }
}
