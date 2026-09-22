use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::{sqlite::SqliteConnectOptions, Connection};

use crate::persistence::{PersistenceError, SqliteSelfModelRepository};

#[derive(Debug)]
pub(crate) enum ArtifactError {
    DestinationExists,
    DestinationUnavailable,
    WriteFailed,
}

#[derive(Debug)]
pub(crate) enum BackupError {
    Storage(PersistenceError),
    Artifact(ArtifactError),
    Validation,
}

/// A destination-local staging directory. Drop removes an unpublished artifact on failure.
pub(crate) struct TempArtifact {
    directory: PathBuf,
    path: PathBuf,
}

impl TempArtifact {
    pub(crate) fn new(final_path: &Path, extension: &str) -> Result<Self, ArtifactError> {
        if final_path.exists() {
            return Err(ArtifactError::DestinationExists);
        }
        let parent = final_path
            .parent()
            .ok_or(ArtifactError::DestinationUnavailable)?;
        if !parent.is_dir() {
            return Err(ArtifactError::DestinationUnavailable);
        }
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ArtifactError::DestinationUnavailable)?
            .as_nanos();
        for attempt in 0..16 {
            let directory = parent.join(format!(
                ".nous-artifact-{}-{nonce}-{attempt}",
                std::process::id()
            ));
            match fs::create_dir(&directory) {
                Ok(()) => {
                    let path = directory.join(format!("artifact.{extension}"));
                    return Ok(Self { directory, path });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Err(ArtifactError::DestinationUnavailable),
            }
        }
        Err(ArtifactError::DestinationUnavailable)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn write_bytes(&self, bytes: &[u8]) -> Result<(), ArtifactError> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.path)
            .map_err(|_| ArtifactError::WriteFailed)?;
        file.write_all(bytes)
            .map_err(|_| ArtifactError::WriteFailed)?;
        file.sync_all().map_err(|_| ArtifactError::WriteFailed)?;
        Ok(())
    }
}

impl Drop for TempArtifact {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

/// Hard-link publication is atomic and fails if the chosen final name already exists.
/// There is deliberately no rename/copy fallback that could overwrite a user's file.
pub(crate) fn publish_artifact(
    artifact: &TempArtifact,
    final_path: &Path,
) -> Result<(), ArtifactError> {
    match fs::hard_link(artifact.path(), final_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(ArtifactError::DestinationExists)
        }
        Err(_) => Err(ArtifactError::DestinationUnavailable),
    }
}

pub(crate) async fn create_database_backup_to_path(
    repository: &SqliteSelfModelRepository,
    destination: &Path,
) -> Result<(), BackupError> {
    let artifact = TempArtifact::new(destination, "sqlite").map_err(BackupError::Artifact)?;
    repository
        .vacuum_into(artifact.path())
        .await
        .map_err(BackupError::Storage)?;
    validate_backup(artifact.path()).await?;
    publish_artifact(&artifact, destination).map_err(BackupError::Artifact)
}

async fn validate_backup(path: &Path) -> Result<(), BackupError> {
    if !path.is_file() {
        return Err(BackupError::Validation);
    }
    let options = SqliteConnectOptions::new().filename(path).read_only(true);
    let mut connection = sqlx::SqliteConnection::connect_with(&options)
        .await
        .map_err(|_| BackupError::Validation)?;
    let result: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&mut connection)
        .await
        .map_err(|_| BackupError::Validation)?;
    if result != "ok" {
        return Err(BackupError::Validation);
    }
    let schema: String =
        sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
            .fetch_one(&mut connection)
            .await
            .map_err(|_| BackupError::Validation)?;
    if schema != "5" {
        return Err(BackupError::Validation);
    }
    let migrations: Vec<(i64, bool)> =
        sqlx::query_as("SELECT version, success FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&mut connection)
            .await
            .map_err(|_| BackupError::Validation)?;
    if migrations != vec![(1, true), (2, true), (3, true), (4, true), (5, true)] {
        return Err(BackupError::Validation);
    }
    let objects: Vec<(String, String)> =
        sqlx::query_as("SELECT type, name FROM sqlite_master WHERE type IN ('table', 'index')")
            .fetch_all(&mut connection)
            .await
            .map_err(|_| BackupError::Validation)?;
    let names: std::collections::HashSet<(&str, &str)> = objects
        .iter()
        .map(|(kind, name)| (kind.as_str(), name.as_str()))
        .collect();
    for table in [
        "app_metadata",
        "_sqlx_migrations",
        "self_subjects",
        "person_references",
        "situations",
        "observations",
        "thoughts",
        "emotions",
        "beliefs",
        "belief_revisions",
        "values",
        "value_revisions",
        "memories",
        "decisions",
        "outcomes",
        "evidence_links",
        "situation_corrections",
        "observation_corrections",
        "thought_corrections",
    ] {
        if !names.contains(&("table", table)) {
            return Err(BackupError::Validation);
        }
    }
    for index in [
        "person_references_subject_id_idx",
        "situations_subject_id_idx",
        "observations_subject_id_idx",
        "observations_situation_subject_idx",
        "thoughts_subject_id_idx",
        "thoughts_situation_subject_idx",
        "emotions_subject_id_idx",
        "emotions_situation_subject_idx",
        "beliefs_subject_id_idx",
        "belief_revisions_belief_id_idx",
        "values_subject_id_idx",
        "value_revisions_value_id_idx",
        "memories_subject_id_idx",
        "memories_situation_subject_idx",
        "decisions_subject_id_idx",
        "decisions_situation_subject_idx",
        "outcomes_subject_id_idx",
        "outcomes_decision_subject_idx",
        "observations_id_subject_unique_idx",
        "thoughts_id_subject_unique_idx",
        "emotions_id_subject_unique_idx",
        "memories_id_subject_unique_idx",
        "outcomes_id_subject_unique_idx",
        "beliefs_id_subject_unique_idx",
        "belief_revisions_id_belief_unique_idx",
        "values_id_subject_unique_idx",
        "value_revisions_id_value_unique_idx",
        "evidence_links_subject_id_idx",
        "evidence_links_source_observation_subject_idx",
        "evidence_links_source_thought_subject_idx",
        "evidence_links_source_emotion_subject_idx",
        "evidence_links_source_situation_subject_idx",
        "evidence_links_source_memory_subject_idx",
        "evidence_links_source_decision_subject_idx",
        "evidence_links_source_outcome_subject_idx",
        "evidence_links_belief_revision_target_idx",
        "evidence_links_value_revision_target_idx",
        "situation_corrections_subject_idx",
        "observation_corrections_subject_idx",
        "observation_corrections_before_situation_subject_idx",
        "observation_corrections_after_situation_subject_idx",
        "thought_corrections_subject_idx",
        "thought_corrections_before_situation_subject_idx",
        "thought_corrections_after_situation_subject_idx",
    ] {
        if !names.contains(&("index", index)) {
            return Err(BackupError::Validation);
        }
    }
    connection
        .close()
        .await
        .map_err(|_| BackupError::Validation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use sqlx::Connection;

    #[test]
    fn vacuum_into_accepts_a_bound_destination_with_sqlx() {
        tauri::async_runtime::block_on(async {
            let source_path = std::env::temp_dir().join(format!(
                "nous-task-011-vacuum-source-{}-{}.sqlite",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(
                    SqliteConnectOptions::new()
                        .filename(&source_path)
                        .create_if_missing(true),
                )
                .await
                .unwrap();
            sqlx::query("CREATE TABLE probe (value TEXT NOT NULL)")
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("INSERT INTO probe (value) VALUES ('bound')")
                .execute(&pool)
                .await
                .unwrap();
            let path = std::env::temp_dir().join(format!(
                "nous-task-011-vacuum-proof-{}-{}.sqlite",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            assert!(!path.exists());
            let fs_probe = path.with_extension("probe");
            std::fs::write(&fs_probe, b"probe").unwrap();
            assert!(fs_probe.exists());
            std::fs::remove_file(fs_probe).unwrap();
            let destination = path.to_str().unwrap().replace('\\', "/");
            let mut connection = pool.acquire().await.unwrap();
            sqlx::query("VACUUM INTO ?")
                .bind(&destination)
                .execute(&mut *connection)
                .await
                .unwrap();
            assert!(path.exists());
            drop(connection);
            let mut backup = sqlx::SqliteConnection::connect_with(
                &SqliteConnectOptions::new().filename(&path).read_only(true),
            )
            .await
            .unwrap();
            let value: String = sqlx::query_scalar("SELECT value FROM probe")
                .fetch_one(&mut backup)
                .await
                .unwrap();
            assert_eq!(value, "bound");
            backup.close().await.unwrap();
            pool.close().await;
            drop(pool);
            std::fs::remove_file(path).unwrap();
            std::fs::remove_file(source_path).unwrap();
        });
    }

    #[test]
    fn publication_never_overwrites_an_existing_or_racing_destination() {
        let directory = std::env::temp_dir().join(format!(
            "nous-task-011-publish-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let destination = directory.join("export.json");
        let artifact = TempArtifact::new(&destination, "json").unwrap();
        artifact.write_bytes(b"complete").unwrap();
        fs::write(&destination, b"existing").unwrap();
        assert!(matches!(
            publish_artifact(&artifact, &destination),
            Err(ArtifactError::DestinationExists)
        ));
        assert_eq!(fs::read(&destination).unwrap(), b"existing");
        drop(artifact);
        fs::remove_file(&destination).unwrap();
        let artifact = TempArtifact::new(&destination, "json").unwrap();
        artifact.write_bytes(b"complete").unwrap();
        publish_artifact(&artifact, &destination).unwrap();
        drop(artifact);
        assert_eq!(fs::read(&destination).unwrap(), b"complete");
        let unsupported = directory.join("missing-parent").join("other.json");
        let artifact = TempArtifact::new(&directory.join("unused.json"), "json").unwrap();
        artifact.write_bytes(b"complete").unwrap();
        assert!(matches!(
            publish_artifact(&artifact, &unsupported),
            Err(ArtifactError::DestinationUnavailable)
        ));
        drop(artifact);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn backup_preserves_schema_and_semantically_invalid_but_sqlite_valid_data() {
        tauri::async_runtime::block_on(async {
            let directory = std::env::temp_dir().join(format!(
                "nous-task-011-backup-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&directory).unwrap();
            let source = directory.join("source.sqlite");
            let pool = SqlitePoolOptions::new()
                .max_connections(2)
                .connect_with(
                    SqliteConnectOptions::new()
                        .filename(&source)
                        .create_if_missing(true)
                        .foreign_keys(true),
                )
                .await
                .unwrap();
            for migration in [
                include_str!("../../migrations/0001_initialize.sql"),
                include_str!("../../migrations/0002_create_self_model.sql"),
                include_str!("../../migrations/0003_create_lived_experience_records.sql"),
                include_str!("../../migrations/0004_create_evidence_links.sql"),
                include_str!("../../migrations/0005_create_structured_corrections.sql"),
            ] {
                sqlx::raw_sql(migration).execute(&pool).await.unwrap();
            }
            sqlx::query("CREATE TABLE _sqlx_migrations (version BIGINT PRIMARY KEY, success BOOLEAN NOT NULL)")
                .execute(&pool).await.unwrap();
            for version in 1..=5 {
                sqlx::query("INSERT INTO _sqlx_migrations VALUES (?, 1)")
                    .bind(version)
                    .execute(&pool)
                    .await
                    .unwrap();
            }
            sqlx::query("INSERT INTO self_subjects VALUES ('self', 'Self', 1)")
                .execute(&pool)
                .await
                .unwrap();
            // SQLite trim accepts NBSP, while the domain's required-text validation rejects it.
            sqlx::query("INSERT INTO person_references VALUES ('p', 'self', ?, 'known', NULL, 2)")
                .bind("\u{00a0}")
                .execute(&pool)
                .await
                .unwrap();
            let repository = SqliteSelfModelRepository::new(
                crate::persistence::SharedSqlitePool::from_test_pool(pool.clone()),
            );
            assert!(matches!(
                repository.load_portable_user_data_snapshot().await,
                Err(PersistenceError::DomainReconstruction { .. })
            ));
            let destination = directory.join("backup.sqlite");
            create_database_backup_to_path(&repository, &destination)
                .await
                .unwrap();
            let backup = SqliteConnectOptions::new()
                .filename(&destination)
                .read_only(true);
            let mut connection = sqlx::SqliteConnection::connect_with(&backup).await.unwrap();
            let copied: String =
                sqlx::query_scalar("SELECT display_name FROM person_references WHERE id = 'p'")
                    .fetch_one(&mut connection)
                    .await
                    .unwrap();
            assert_eq!(copied, "\u{00a0}");
            connection.close().await.unwrap();
            sqlx::query("DELETE FROM person_references WHERE id = 'p'")
                .execute(&pool)
                .await
                .unwrap();
            let after_delete = directory.join("after-delete.sqlite");
            create_database_backup_to_path(&repository, &after_delete)
                .await
                .unwrap();
            let options = SqliteConnectOptions::new()
                .filename(&after_delete)
                .read_only(true);
            let mut second = sqlx::SqliteConnection::connect_with(&options)
                .await
                .unwrap();
            let count: i64 = sqlx::query_scalar("SELECT count(*) FROM person_references")
                .fetch_one(&mut second)
                .await
                .unwrap();
            assert_eq!(count, 0);
            second.close().await.unwrap();
            drop(repository);
            pool.close().await;
            drop(pool);
            fs::remove_dir_all(directory).unwrap();
        });
    }
}
