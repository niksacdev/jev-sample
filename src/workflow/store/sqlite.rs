use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use rusqlite::{Connection, OptionalExtension, params};

use super::{Admission, RepositoryFuture, Snapshot, StoreError, WorkflowRepository, parse_id};
use crate::workflow::{contracts::WorkflowEvent, policy::WorkflowPolicy};

const LATEST_MIGRATION: i64 = 2;

pub struct SqliteRepository {
    connection: Arc<Mutex<Connection>>,
    max_records: u32,
    max_bytes: u64,
}

impl SqliteRepository {
    pub async fn open(path: &Path, policy: &WorkflowPolicy) -> Result<Self, StoreError> {
        let path = path.to_owned();
        let max_pages = policy.max_storage_bytes / 4096;
        let connection = tokio::task::spawn_blocking(move || -> Result<Connection, StoreError> {
            secure_path(&path)?;
            let connection = Connection::open(path).map_err(|_| StoreError)?;
            connection
                .busy_timeout(std::time::Duration::from_secs(2))
                .map_err(|_| StoreError)?;
            connection
                .execute_batch(&format!(
                    "PRAGMA page_size=4096; PRAGMA max_page_count={max_pages};"
                ))
                .map_err(|_| StoreError)?;
            let pages: u32 = connection
                .query_row("PRAGMA max_page_count", [], |row| row.get(0))
                .map_err(|_| StoreError)?;
            if u64::from(pages) > max_pages {
                return Err(StoreError);
            }
            connection
                .execute_batch(
                    "PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL; PRAGMA secure_delete=ON;",
                )
                .map_err(|_| StoreError)?;
            migrate(&connection)?;
            Ok(connection)
        })
        .await
        .map_err(|_| StoreError)??;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            max_records: policy.max_comparisons,
            max_bytes: policy.max_storage_bytes,
        })
    }

    async fn sql<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&mut Connection) -> Result<T, StoreError> + Send + 'static,
    ) -> Result<T, StoreError> {
        let connection = self.connection.clone();
        tokio::task::spawn_blocking(move || {
            let mut guard = connection.lock().map_err(|_| StoreError)?;
            operation(&mut guard)
        })
        .await
        .map_err(|_| StoreError)?
    }
}

impl WorkflowRepository for SqliteRepository {
    fn admit<'a>(
        &'a self,
        request: String,
        payload: String,
        mut snapshot: Snapshot,
    ) -> RepositoryFuture<'a, Admission> {
        let max_records = self.max_records;
        let max_bytes = self.max_bytes;
        Box::pin(async move {
            self.sql(move |connection| {
                let transaction = connection.transaction().map_err(|_| StoreError)?;
                if let Some((old, id)) = transaction
                    .query_row(
                        "SELECT payload_key,comparison_id FROM admissions WHERE request_key=?1",
                        [&request],
                        |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
                    )
                    .optional()
                    .map_err(|_| StoreError)?
                {
                    if old != payload {
                        return Err(StoreError);
                    }
                    let json: String = transaction
                        .query_row("SELECT snapshot FROM comparisons WHERE id=?1", [id], |r| r.get(0))
                        .map_err(|_| StoreError)?;
                    return Ok(Admission::Existing(Box::new(
                        serde_json::from_str(&json).map_err(|_| StoreError)?,
                    )));
                }
                let count: u32 = transaction
                    .query_row("SELECT count(*) FROM comparisons", [], |r| r.get(0))
                    .map_err(|_| StoreError)?;
                let bytes: i64 = transaction
                    .query_row("SELECT coalesce(sum(snapshot_bytes),0) FROM comparisons", [], |r| r.get(0))
                    .map_err(|_| StoreError)?;
                let admission_count: u32 = transaction
                    .query_row("SELECT count(*) FROM admissions", [], |r| r.get(0))
                    .map_err(|_| StoreError)?;
                if count >= max_records || u64::try_from(bytes).map_err(|_| StoreError)? >= max_bytes
                    || admission_count >= max_records * 64
                {
                    return Err(StoreError);
                }
                let id: i64 = transaction
                    .query_row("INSERT INTO comparisons(snapshot, snapshot_bytes) VALUES('', 0) RETURNING id", [], |r| r.get(0))
                    .map_err(|_| StoreError)?;
                snapshot.detail.comparison.comparison_id = format!("comparison-{id}");
                for (index, run) in snapshot.detail.comparison.runs.iter_mut().enumerate() {
                    run.run_id = format!("comparison-{id}-run-{}", index + 1);
                }
                let json = serde_json::to_string(&snapshot).map_err(|_| StoreError)?;
                let snapshot_bytes = i64::try_from(json.len()).map_err(|_| StoreError)?;
                if u64::try_from(bytes).map_err(|_| StoreError)? + json.len() as u64 > max_bytes {
                    return Err(StoreError);
                }
                transaction
                    .execute("UPDATE comparisons SET snapshot=?1, snapshot_bytes=?2 WHERE id=?3", params![json, snapshot_bytes, id])
                    .map_err(|_| StoreError)?;
                transaction
                    .execute("INSERT INTO admissions(request_key,payload_key,comparison_id) VALUES(?1,?2,?3)", params![request, payload, id])
                    .map_err(|_| StoreError)?;
                transaction.commit().map_err(|_| StoreError)?;
                Ok(Admission::New(format!("comparison-{id}")))
            }).await
        })
    }

    fn reserve_resume<'a>(
        &'a self,
        mut snapshot: Snapshot,
        request: String,
        payload: String,
    ) -> RepositoryFuture<'a, bool> {
        let id = match parse_id(&snapshot.detail.comparison.comparison_id) {
            Ok(id) => id,
            Err(error) => return Box::pin(async move { Err(error) }),
        };
        let cap = u64::from(self.max_records) * 64;
        let max_bytes = self.max_bytes;
        Box::pin(async move {
            self.sql(move |connection| {
                let tx = connection.transaction().map_err(|_| StoreError)?;
                let old: Option<(String, i64)> = tx
                    .query_row("SELECT payload_key,comparison_id FROM admissions WHERE request_key=?1", [&request], |r| Ok((r.get(0)?, r.get(1)?)))
                    .optional().map_err(|_| StoreError)?;
                if let Some((old, previous)) = old {
                    return if old == payload && previous == id { Ok(false) } else { Err(StoreError) };
                }
                let count: u32 = tx.query_row("SELECT count(*) FROM admissions", [], |r| r.get(0)).map_err(|_| StoreError)?;
                if u64::from(count) >= cap { return Err(StoreError); }
                tx.execute("INSERT INTO admissions(request_key,payload_key,comparison_id) VALUES(?1,?2,?3)", params![request, payload, id]).map_err(|_| StoreError)?;
                stamp_events(&tx, &mut snapshot, id)?;
                let json = serde_json::to_string(&snapshot).map_err(|_| StoreError)?;
                let snapshot_bytes = i64::try_from(json.len()).map_err(|_| StoreError)?;
                let other_bytes: i64 = tx.query_row("SELECT coalesce(sum(snapshot_bytes),0) FROM comparisons WHERE id!=?1", [id], |r| r.get(0)).map_err(|_| StoreError)?;
                if u64::try_from(other_bytes).map_err(|_| StoreError)? + json.len() as u64 > max_bytes { return Err(StoreError); }
                tx.execute("UPDATE comparisons SET snapshot=?1, snapshot_bytes=?2 WHERE id=?3", params![json, snapshot_bytes, id]).map_err(|_| StoreError)?;
                tx.commit().map_err(|_| StoreError)?;
                Ok(true)
            }).await
        })
    }

    fn prior_request<'a>(
        &'a self,
        request: String,
        payload: String,
        comparison: String,
    ) -> RepositoryFuture<'a, bool> {
        let id = match parse_id(&comparison) {
            Ok(id) => id,
            Err(error) => return Box::pin(async move { Err(error) }),
        };
        Box::pin(async move {
            self.sql(move |c| {
                let prior: Option<(String, i64)> = c
                    .query_row(
                        "SELECT payload_key,comparison_id FROM admissions WHERE request_key=?1",
                        [request],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .optional()
                    .map_err(|_| StoreError)?;
                match prior {
                    Some((key, prior_id)) if key == payload && prior_id == id => Ok(true),
                    Some(_) => Err(StoreError),
                    None => Ok(false),
                }
            })
            .await
        })
    }

    fn save<'a>(&'a self, mut snapshot: Snapshot) -> RepositoryFuture<'a, ()> {
        let id = match parse_id(&snapshot.detail.comparison.comparison_id) {
            Ok(id) => id,
            Err(error) => return Box::pin(async move { Err(error) }),
        };
        let max = self.max_bytes;
        Box::pin(async move {
            self.sql(move |connection| {
                let tx = connection.transaction().map_err(|_| StoreError)?;
                stamp_events(&tx, &mut snapshot, id)?;
                let json = serde_json::to_string(&snapshot).map_err(|_| StoreError)?;
                let snapshot_bytes = i64::try_from(json.len()).map_err(|_| StoreError)?;
                let bytes: i64 = tx
                    .query_row(
                        "SELECT coalesce(sum(snapshot_bytes),0) FROM comparisons WHERE id!=?1",
                        [id],
                        |r| r.get(0),
                    )
                    .map_err(|_| StoreError)?;
                if u64::try_from(bytes).map_err(|_| StoreError)? + json.len() as u64 > max {
                    return Err(StoreError);
                }
                if tx
                    .execute(
                        "UPDATE comparisons SET snapshot=?1, snapshot_bytes=?2 WHERE id=?3",
                        params![json, snapshot_bytes, id],
                    )
                    .map_err(|_| StoreError)?
                    != 1
                {
                    return Err(StoreError);
                }
                tx.commit().map_err(|_| StoreError)
            })
            .await
        })
    }

    fn get<'a>(&'a self, id: String) -> RepositoryFuture<'a, Snapshot> {
        let id = match parse_id(&id) {
            Ok(id) => id,
            Err(error) => return Box::pin(async move { Err(error) }),
        };
        Box::pin(async move {
            self.sql(move |c| {
                let json: String = c
                    .query_row("SELECT snapshot FROM comparisons WHERE id=?1", [id], |r| {
                        r.get(0)
                    })
                    .map_err(|_| StoreError)?;
                serde_json::from_str(&json).map_err(|_| StoreError)
            })
            .await
        })
    }

    fn list<'a>(&'a self) -> RepositoryFuture<'a, Vec<Snapshot>> {
        Box::pin(async move {
            self.sql(|c| {
                let mut query = c
                    .prepare("SELECT snapshot FROM comparisons ORDER BY id DESC")
                    .map_err(|_| StoreError)?;
                let rows = query
                    .query_map([], |r| r.get::<_, String>(0))
                    .map_err(|_| StoreError)?;
                rows.map(|r| {
                    serde_json::from_str(&r.map_err(|_| StoreError)?).map_err(|_| StoreError)
                })
                .collect()
            })
            .await
        })
    }
}

fn migrate(connection: &Connection) -> Result<(), StoreError> {
    connection.execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations(version INTEGER PRIMARY KEY, applied_at_ms INTEGER NOT NULL);").map_err(|_| StoreError)?;
    let versions = {
        let mut statement = connection
            .prepare("SELECT version FROM schema_migrations ORDER BY version")
            .map_err(|_| StoreError)?;
        statement
            .query_map([], |row| row.get::<_, i64>(0))
            .map_err(|_| StoreError)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| StoreError)?
    };
    if versions
        .iter()
        .any(|version| *version > LATEST_MIGRATION || *version < 1)
    {
        return Err(StoreError);
    }
    for (version, migration) in [
        (
            1,
            include_str!("../../../migrations/sqlite/0001_initial.sql"),
        ),
        (
            2,
            include_str!("../../../migrations/sqlite/0002_snapshot_bytes.sql"),
        ),
    ] {
        if !versions.contains(&version) {
            let tx = connection.unchecked_transaction().map_err(|_| StoreError)?;
            tx.execute_batch(migration).map_err(|_| StoreError)?;
            tx.execute(
                "INSERT INTO schema_migrations(version,applied_at_ms) VALUES(?1,?2)",
                params![version, super::epoch_ms()? as i64],
            )
            .map_err(|_| StoreError)?;
            tx.commit().map_err(|_| StoreError)?;
        }
    }
    Ok(())
}

fn stamp_events(
    tx: &rusqlite::Transaction<'_>,
    snapshot: &mut Snapshot,
    id: i64,
) -> Result<(), StoreError> {
    for run in &mut snapshot.detail.comparison.runs {
        let count: u32 = tx
            .query_row(
                "SELECT count(*) FROM events WHERE comparison_id=?1 AND run_id=?2",
                params![id, run.run_id],
                |r| r.get(0),
            )
            .map_err(|_| StoreError)?;
        if count as usize > run.event_trace.len() {
            return Err(StoreError);
        }
        for (index, event) in run.event_trace.iter_mut().enumerate() {
            if event.sequence as usize != index + 1
                || event.run_id != run.run_id
                || event.comparison_id != snapshot.detail.comparison.comparison_id
            {
                return Err(StoreError);
            }
            let prior: Option<String> = tx
                .query_row(
                    "SELECT event FROM events WHERE comparison_id=?1 AND run_id=?2 AND sequence=?3",
                    params![id, run.run_id, event.sequence],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|_| StoreError)?;
            if let Some(prior) = prior {
                let stored: WorkflowEvent = serde_json::from_str(&prior).map_err(|_| StoreError)?;
                if event.recorded_at_ms.is_some() && event.recorded_at_ms != stored.recorded_at_ms {
                    return Err(StoreError);
                }
                event.recorded_at_ms = stored.recorded_at_ms;
                if serde_json::to_string(event).map_err(|_| StoreError)? != prior {
                    return Err(StoreError);
                }
            } else {
                event.recorded_at_ms = Some(super::epoch_ms()?);
                let encoded = serde_json::to_string(event).map_err(|_| StoreError)?;
                tx.execute(
                    "INSERT INTO events(comparison_id,run_id,sequence,event) VALUES(?1,?2,?3,?4)",
                    params![id, run.run_id, event.sequence, encoded],
                )
                .map_err(|_| StoreError)?;
            }
        }
    }
    Ok(())
}

fn secure_path(path: &Path) -> Result<(), StoreError> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or(StoreError)?;
    if !parent.exists() {
        std::fs::create_dir(parent).map_err(|_| StoreError)?;
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
            .map_err(|_| StoreError)?;
    }
    let meta = std::fs::symlink_metadata(parent).map_err(|_| StoreError)?;
    if meta.file_type().is_symlink() || !meta.is_dir() || meta.permissions().mode() & 0o077 != 0 {
        return Err(StoreError);
    }
    if path.exists() || std::fs::symlink_metadata(path).is_ok() {
        let meta = std::fs::symlink_metadata(path).map_err(|_| StoreError)?;
        if !meta.is_file()
            || meta.file_type().is_symlink()
            || meta.permissions().mode() & 0o077 != 0
        {
            return Err(StoreError);
        }
    } else {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|_| StoreError)?;
    }
    let journal = PathBuf::from(format!("{}-journal", path.to_str().ok_or(StoreError)?));
    if let Ok(meta) = std::fs::symlink_metadata(journal)
        && (!meta.is_file()
            || meta.file_type().is_symlink()
            || meta.permissions().mode() & 0o077 != 0)
    {
        return Err(StoreError);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::workflow::{
        contracts::{
            OperatorWorkflow, ProviderId, WorkflowComparison, WorkflowPlan, WorkflowRun,
            WorkflowUsage,
        },
        store::{WorkflowStore, repository_contract},
    };
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct TestDir(PathBuf);
    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "zipclaim-store-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir(&path).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
            Self(path)
        }
        fn db(&self) -> PathBuf {
            self.0.join("workflow.sqlite")
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn fixture() -> Snapshot {
        Snapshot {
            detail: OperatorWorkflow {
                comparison: WorkflowComparison {
                    comparison_id: String::new(),
                    input_key: "key".into(),
                    base_plan: WorkflowPlan {
                        plan_id: "plan".into(),
                        tasks: vec![],
                    },
                    runs: vec![WorkflowRun {
                        run_id: String::new(),
                        plan_id: "plan".into(),
                        provider: ProviderId::Code,
                        model: None,
                        state: crate::workflow::contracts::WorkflowState::Completed,
                        reply: "done".into(),
                        tasks: vec![],
                        event_trace: vec![],
                        usage: WorkflowUsage::default(),
                        elapsed_ms: 0,
                        failure_code: None,
                    }],
                    mode: "test".into(),
                    complete: true,
                    planner_model: None,
                    planner_usage: WorkflowUsage::default(),
                    failure_code: None,
                },
                message: "fixture".into(),
                decisions: vec![],
                protected_context_json: "{}".into(),
            },
            policy: WorkflowPolicy::from_json(include_str!("../../../config/workflow.json"))
                .unwrap(),
            context: "context".into(),
            run_contexts: Default::default(),
        }
    }
    fn policy() -> WorkflowPolicy {
        WorkflowPolicy::from_json(include_str!("../../../config/workflow.json")).unwrap()
    }

    #[tokio::test]
    async fn sqlite_satisfies_repository_contract() {
        let dir = TestDir::new();
        let mut contract_policy = policy();
        contract_policy.max_comparisons = 2;
        let store = WorkflowStore::open(&dir.db(), &contract_policy)
            .await
            .unwrap();
        repository_contract(store).await;
    }

    #[tokio::test]
    async fn migration_backfills_existing_snapshot_bytes() {
        let dir = TestDir::new();
        let db_path = dir.db();
        let store = WorkflowStore::open(&db_path, &policy()).await.unwrap();
        store
            .admit("request".into(), "payload".into(), fixture())
            .await
            .unwrap();
        drop(store);
        let connection = Connection::open(&db_path).unwrap();
        connection.execute_batch("ALTER TABLE comparisons RENAME TO comparisons_with_bytes; CREATE TABLE comparisons(id INTEGER PRIMARY KEY, snapshot TEXT NOT NULL); INSERT INTO comparisons(id,snapshot) SELECT id,snapshot FROM comparisons_with_bytes; DROP TABLE comparisons_with_bytes; DELETE FROM schema_migrations WHERE version=2;").unwrap();
        let expected: i64 = connection
            .query_row(
                "SELECT length(CAST(snapshot AS BLOB)) FROM comparisons",
                [],
                |r| r.get(0),
            )
            .unwrap();
        drop(connection);
        let migrated = SqliteRepository::open(&db_path, &policy()).await.unwrap();
        drop(migrated);
        let connection = Connection::open(db_path).unwrap();
        let actual: i64 = connection
            .query_row("SELECT snapshot_bytes FROM comparisons", [], |r| r.get(0))
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn unknown_migration_version_is_rejected() {
        let dir = TestDir::new();
        let connection = Connection::open(dir.db()).unwrap();
        connection.execute_batch("CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, applied_at_ms INTEGER NOT NULL); INSERT INTO schema_migrations VALUES(99,0);").unwrap();
        drop(connection);
        assert!(WorkflowStore::open(&dir.db(), &policy()).await.is_err());
    }

    use std::os::unix::fs::PermissionsExt;
}
