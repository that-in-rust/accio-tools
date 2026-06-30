use std::path::Path;
use std::sync::Mutex;

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::{model_label_text, PieError, PromptVersion, VersionKind};

pub struct SqliteVersionStore {
    connection: Mutex<Connection>,
}

impl SqliteVersionStore {
    pub fn open_in_memory() -> Result<Self, PieError> {
        let connection = Connection::open_in_memory()?;
        let store = Self {
            connection: Mutex::new(connection),
        };
        store.initialize_schema()?;
        Ok(store)
    }

    pub fn open_at_path(path: impl AsRef<Path>) -> Result<Self, PieError> {
        Self::open_store_at_path(path)
    }

    pub fn open_store_at_path(path: impl AsRef<Path>) -> Result<Self, PieError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)?;
        let store = Self {
            connection: Mutex::new(connection),
        };
        store.initialize_schema()?;
        Ok(store)
    }

    pub fn store_prompt_version(
        &self,
        filename: &str,
        kind: VersionKind,
        payload: &str,
        analysis_run_id: &str,
        model_label: String,
    ) -> Result<PromptVersion, PieError> {
        let version_name = create_version_name(filename, &kind);
        let payload_size = payload.len();
        let connection = self.connection.lock().map_err(|_| PieError::Sqlite {
            message: "version store lock poisoned".to_string(),
        })?;

        connection.execute(
            "insert into prompt_versions
                (version_name, kind, payload, payload_size, analysis_run_id, model_label)
             values (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                version_name,
                kind.as_label(),
                payload,
                payload_size as i64,
                analysis_run_id,
                model_label
            ],
        )?;

        Ok(PromptVersion {
            version_name,
            kind,
            payload_size,
            analysis_run_id: analysis_run_id.to_string(),
            model_label: model_label_text(),
        })
    }

    pub fn load_version_payload(&self, version_name: &str) -> Result<String, PieError> {
        let connection = self.connection.lock().map_err(|_| PieError::Sqlite {
            message: "version store lock poisoned".to_string(),
        })?;
        let payload = connection.query_row(
            "select payload from prompt_versions where version_name = ?1",
            [version_name],
            |row| row.get::<_, String>(0),
        )?;
        Ok(payload)
    }

    fn initialize_schema(&self) -> Result<(), PieError> {
        let connection = self.connection.lock().map_err(|_| PieError::Sqlite {
            message: "version store lock poisoned".to_string(),
        })?;
        connection.execute_batch(
            "create table if not exists prompt_versions (
                version_name text primary key,
                kind text not null,
                payload text not null,
                payload_size integer not null,
                analysis_run_id text not null,
                model_label text not null,
                created_at text not null default current_timestamp
            );",
        )?;
        Ok(())
    }
}

fn create_version_name(filename: &str, kind: &VersionKind) -> String {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S%3f");
    let base = match kind {
        VersionKind::Original => Path::new(filename)
            .file_stem()
            .and_then(|value| value.to_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("prompt"),
        VersionKind::Updated => "updated_prompt",
    };
    format!("{base}_v{timestamp}")
}
