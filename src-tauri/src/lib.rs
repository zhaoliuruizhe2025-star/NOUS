use tauri_plugin_sql::{Migration, MigrationKind};

mod application;
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
        Migration {
            version: 3,
            description: "create_lived_experience_records",
            sql: include_str!("../migrations/0003_create_lived_experience_records.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "create_evidence_links",
            sql: include_str!("../migrations/0004_create_evidence_links.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "create_structured_corrections",
            sql: include_str!("../migrations/0005_create_structured_corrections.sql"),
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
        .invoke_handler(tauri::generate_handler![
            commands::database_status,
            commands::save_structured_capture,
            commands::load_structured_history,
            commands::correct_structured_record,
            commands::delete_structured_record
        ])
        .run(tauri::generate_context!())
        .expect("error while running NOUS");
}

#[cfg(test)]
mod tests {
    use super::migrations;

    #[test]
    fn contains_the_registered_task_migrations() {
        let migrations = migrations();

        assert_eq!(migrations.len(), 5);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[0].description, "initialize_local_storage");
        assert!(migrations[0].sql.contains("app_metadata"));
        assert_eq!(migrations[1].version, 2);
        assert_eq!(migrations[1].description, "create_self_model");
        assert!(migrations[1].sql.contains("CREATE TABLE \"values\""));
        assert_eq!(migrations[2].version, 3);
        assert_eq!(migrations[2].description, "create_lived_experience_records");
        assert!(migrations[2].sql.contains("CREATE TABLE memories"));
        assert!(migrations[2].sql.contains("CREATE TABLE decisions"));
        assert!(migrations[2].sql.contains("CREATE TABLE outcomes"));
        assert_eq!(migrations[3].version, 4);
        assert_eq!(migrations[3].description, "create_evidence_links");
        assert!(migrations[3].sql.contains("CREATE TABLE evidence_links"));
        assert_eq!(migrations[4].version, 5);
        assert_eq!(migrations[4].description, "create_structured_corrections");
        assert!(migrations[4]
            .sql
            .contains("CREATE TABLE thought_corrections"));
    }
}
