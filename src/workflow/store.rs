use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    contracts::{OperatorWorkflow, WorkflowState},
    policy::WorkflowPolicy,
};

pub fn identity(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Snapshot {
    pub detail: OperatorWorkflow,
    pub policy: WorkflowPolicy,
    pub context: String,
    pub run_contexts: std::collections::BTreeMap<String, String>,
}

#[derive(Clone)]
pub struct WorkflowStore {
    connection: Arc<Mutex<Connection>>,
    max_records: u32,
    max_bytes: u64,
}

#[derive(Debug)]
pub struct StoreError;
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("workflow storage failed or exhausted")
    }
}
impl std::error::Error for StoreError {}

pub enum Admission {
    New(String),
    Existing(Box<Snapshot>),
}

impl WorkflowStore {
    pub async fn open(path: &Path, policy: &WorkflowPolicy) -> Result<Self, StoreError> {
        let path = path.to_owned();
        let max_pages = policy.max_storage_bytes / 4096;
        let connection = tokio::task::spawn_blocking(move || -> Result<Connection, StoreError> {
            secure_path(&path)?;
            let connection = Connection::open(path).map_err(|_|StoreError)?;
            connection.busy_timeout(std::time::Duration::from_secs(2)).map_err(|_|StoreError)?;
            connection.execute_batch(&format!("PRAGMA page_size=4096; PRAGMA max_page_count={max_pages};")).map_err(|_|StoreError)?;
            let pages:u32=connection.query_row("PRAGMA max_page_count",[],|row|row.get(0)).map_err(|_|StoreError)?;
            if u64::from(pages)>max_pages { return Err(StoreError); }
            connection.execute_batch(
                "PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL; PRAGMA secure_delete=ON;
                 CREATE TABLE IF NOT EXISTS comparisons(id INTEGER PRIMARY KEY, snapshot TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS admissions(request_key TEXT PRIMARY KEY, payload_key TEXT NOT NULL, comparison_id INTEGER NOT NULL);
                 CREATE TABLE IF NOT EXISTS events(comparison_id INTEGER NOT NULL, run_id TEXT NOT NULL, sequence INTEGER NOT NULL, event TEXT NOT NULL, PRIMARY KEY(comparison_id,run_id,sequence));"
            ).map_err(|_|StoreError)?;
            Ok(connection)
        }).await.map_err(|_|StoreError)??;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
            max_records: policy.max_comparisons,
            max_bytes: policy.max_storage_bytes,
        };
        for mut snapshot in store.list().await? {
            let mut changed = false;
            for run in &mut snapshot.detail.comparison.runs {
                if run.state == WorkflowState::Running {
                    changed = true;
                    run.state = WorkflowState::Interrupted;
                    run.failure_code = Some("process_interrupted".into());
                    run.reply = "The process stopped before the workflow completed. No automatic replay was performed.".into();
                    let now = epoch_ms()?;
                    run.event_trace.push(super::contracts::WorkflowEvent {
                        sequence: u32::try_from(run.event_trace.len() + 1)
                            .map_err(|_| StoreError)?,
                        comparison_id: snapshot.detail.comparison.comparison_id.clone(),
                        run_id: run.run_id.clone(),
                        task_id: None,
                        provider: None,
                        model: None,
                        actor: "runtime".into(),
                        plan_id: (!run.plan_id.is_empty()).then(|| run.plan_id.clone()),
                        policy_version: snapshot.policy.version.clone(),
                        schema_version: "workflow-events-1".into(),
                        stage: "workflow_interrupted".into(),
                        outcome: "interrupted".into(),
                        occurred_at_ms: now,
                        recorded_at_ms: None,
                    });
                }
            }
            if snapshot.detail.comparison.base_plan.tasks.is_empty()
                && snapshot.detail.comparison.failure_code.is_none()
            {
                snapshot.detail.comparison.failure_code = Some("planning_interrupted".into());
                changed = true;
            }
            if changed {
                snapshot.detail.comparison.complete = false;
                store.save(snapshot).await?;
            }
        }
        Ok(store)
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

    pub async fn admit(
        &self,
        request: String,
        payload: String,
        mut snapshot: Snapshot,
    ) -> Result<Admission, StoreError> {
        let max_records = self.max_records;
        let max_bytes = self.max_bytes;
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
                    .query_row("SELECT snapshot FROM comparisons WHERE id=?1", [id], |r| {
                        r.get(0)
                    })
                    .map_err(|_| StoreError)?;
                return Ok(Admission::Existing(Box::new(
                    serde_json::from_str(&json).map_err(|_| StoreError)?,
                )));
            }
            let count: u32 = transaction
                .query_row("SELECT count(*) FROM comparisons", [], |r| r.get(0))
                .map_err(|_| StoreError)?;
            let bytes: u32 = transaction
                .query_row(
                    "SELECT coalesce(sum(length(CAST(snapshot AS BLOB))),0) FROM comparisons",
                    [],
                    |r| r.get(0),
                )
                .map_err(|_| StoreError)?;
            let admission_count: u32 = transaction
                .query_row("SELECT count(*) FROM admissions", [], |r| r.get(0))
                .map_err(|_| StoreError)?;
            if count >= max_records
                || u64::from(bytes) >= max_bytes
                || admission_count >= max_records * 64
            {
                return Err(StoreError);
            }
            transaction
                .execute("INSERT INTO comparisons(snapshot) VALUES('')", [])
                .map_err(|_| StoreError)?;
            let id = transaction.last_insert_rowid();
            snapshot.detail.comparison.comparison_id = format!("comparison-{id}");
            for (index, run) in snapshot.detail.comparison.runs.iter_mut().enumerate() {
                run.run_id = format!("comparison-{id}-run-{}", index + 1);
            }
            let json = serde_json::to_string(&snapshot).map_err(|_| StoreError)?;
            if u64::from(bytes) + json.len() as u64 > max_bytes {
                return Err(StoreError);
            }
            transaction
                .execute(
                    "UPDATE comparisons SET snapshot=?1 WHERE id=?2",
                    params![json, id],
                )
                .map_err(|_| StoreError)?;
            transaction
                .execute(
                    "INSERT INTO admissions VALUES(?1,?2,?3)",
                    params![request, payload, id],
                )
                .map_err(|_| StoreError)?;
            transaction.commit().map_err(|_| StoreError)?;
            Ok(Admission::New(format!("comparison-{id}")))
        })
        .await
    }

    pub async fn reserve_resume(
        &self,
        mut snapshot: Snapshot,
        request: String,
        payload: String,
    ) -> Result<bool, StoreError> {
        let id = parse_id(&snapshot.detail.comparison.comparison_id)?;
        let cap = u64::from(self.max_records) * 64;
        self.sql(move |connection| {
            let tx = connection.transaction().map_err(|_| StoreError)?;
            let old: Option<(String, i64)> = tx
                .query_row(
                    "SELECT payload_key,comparison_id FROM admissions WHERE request_key=?1",
                    [&request],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()
                .map_err(|_| StoreError)?;
            if let Some((old, previous)) = old {
                return if old == payload && previous == id {
                    Ok(false)
                } else {
                    Err(StoreError)
                };
            }
            let count: u32 = tx
                .query_row("SELECT count(*) FROM admissions", [], |r| r.get(0))
                .map_err(|_| StoreError)?;
            if u64::from(count) >= cap {
                return Err(StoreError);
            }
            tx.execute(
                "INSERT INTO admissions VALUES(?1,?2,?3)",
                params![request, payload, id],
            )
            .map_err(|_| StoreError)?;
            stamp_events(&tx, &mut snapshot, id)?;
            let json = serde_json::to_string(&snapshot).map_err(|_| StoreError)?;
            tx.execute(
                "UPDATE comparisons SET snapshot=?1 WHERE id=?2",
                params![json, id],
            )
            .map_err(|_| StoreError)?;
            tx.commit().map_err(|_| StoreError)?;
            Ok(true)
        })
        .await
    }

    pub async fn prior_request(
        &self,
        request: String,
        payload: String,
        comparison: String,
    ) -> Result<bool, StoreError> {
        let id = parse_id(&comparison)?;
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
    }

    pub async fn save(&self, mut snapshot: Snapshot) -> Result<(), StoreError> {
        let id = parse_id(&snapshot.detail.comparison.comparison_id)?;
        let max = self.max_bytes;
        self.sql(move |connection| {
            let tx=connection.transaction().map_err(|_|StoreError)?;
            stamp_events(&tx,&mut snapshot,id)?;
            let json=serde_json::to_string(&snapshot).map_err(|_|StoreError)?;
            let bytes:u32=tx.query_row("SELECT coalesce(sum(length(CAST(snapshot AS BLOB))),0) FROM comparisons WHERE id!=?1",[id],|r|r.get(0)).map_err(|_|StoreError)?;
            if u64::from(bytes)+json.len() as u64>max {return Err(StoreError);}
            if tx.execute("UPDATE comparisons SET snapshot=?1 WHERE id=?2",params![json,id]).map_err(|_|StoreError)?!=1 {return Err(StoreError);}
            tx.commit().map_err(|_|StoreError)
        }).await
    }

    pub async fn get(&self, id: String) -> Result<Snapshot, StoreError> {
        let id = parse_id(&id)?;
        self.sql(move |c| {
            let json: String = c
                .query_row("SELECT snapshot FROM comparisons WHERE id=?1", [id], |r| {
                    r.get(0)
                })
                .map_err(|_| StoreError)?;
            serde_json::from_str(&json).map_err(|_| StoreError)
        })
        .await
    }
    pub async fn list(&self) -> Result<Vec<Snapshot>, StoreError> {
        self.sql(|c| {
            let mut query = c
                .prepare("SELECT snapshot FROM comparisons ORDER BY id DESC")
                .map_err(|_| StoreError)?;
            let rows = query
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|_| StoreError)?;
            rows.map(|r| serde_json::from_str(&r.map_err(|_| StoreError)?).map_err(|_| StoreError))
                .collect()
        })
        .await
    }
}

pub fn epoch_ms() -> Result<f64, StoreError> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| StoreError)?
        .as_millis() as f64)
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
                let stored: super::contracts::WorkflowEvent =
                    serde_json::from_str(&prior).map_err(|_| StoreError)?;
                if event.recorded_at_ms.is_some() && event.recorded_at_ms != stored.recorded_at_ms {
                    return Err(StoreError);
                }
                event.recorded_at_ms = stored.recorded_at_ms;
                if serde_json::to_string(event).map_err(|_| StoreError)? != prior {
                    return Err(StoreError);
                }
            } else {
                event.recorded_at_ms = Some(epoch_ms()?);
                let encoded = serde_json::to_string(event).map_err(|_| StoreError)?;
                tx.execute(
                    "INSERT INTO events VALUES(?1,?2,?3,?4)",
                    params![id, run.run_id, event.sequence, encoded],
                )
                .map_err(|_| StoreError)?;
            }
        }
    }
    Ok(())
}
fn parse_id(id: &str) -> Result<i64, StoreError> {
    id.strip_prefix("comparison-")
        .and_then(|s| s.parse().ok())
        .filter(|id| *id > 0)
        .ok_or(StoreError)
}
fn secure_path(path: &Path) -> Result<(), StoreError> {
    use std::os::unix::fs::PermissionsExt;
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
    if meta.file_type().is_symlink() || !meta.is_dir() {
        return Err(StoreError);
    }
    if meta.permissions().mode() & 0o077 != 0 {
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
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|_| StoreError)?;
    }
    let journal = std::path::PathBuf::from(format!("{}-journal", path.to_str().ok_or(StoreError)?));
    if let Ok(meta) = std::fs::symlink_metadata(journal)
        && (!meta.is_file()
            || meta.file_type().is_symlink()
            || meta.permissions().mode() & 0o077 != 0)
    {
        return Err(StoreError);
    }
    Ok(())
}
