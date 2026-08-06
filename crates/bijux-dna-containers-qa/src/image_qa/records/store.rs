use std::path::{Path, PathBuf};

use anyhow::Context;
use anyhow::Result;
use bijux_dna_analyze::load::sqlite::bench::{ensure_image_qa_tables, insert_image_qa_v1};
use bijux_dna_analyze::load::sqlite::reports::insert_image_qa_input_v1;
use bijux_dna_analyze::{append_image_qa_jsonl, open_sqlite, ImageQaRecord};
use bijux_dna_infra::atomic_write_bytes;

use crate::api::PlatformSpec;
use crate::image_qa::support::{image_qa_jsonl_path, image_qa_sqlite_path};

pub(crate) struct QaRecordStore {
    conn: rusqlite::Connection,
    qa_jsonl: PathBuf,
    qa_summary: PathBuf,
    platform_name: String,
    runner_name: String,
}

impl QaRecordStore {
    pub(crate) fn prepare(cwd: &Path, platform: &PlatformSpec) -> Result<Self> {
        let qa_jsonl = image_qa_jsonl_path(cwd, &platform.name);
        let qa_sqlite = image_qa_sqlite_path(cwd, &platform.name);
        let qa_dir = qa_sqlite.parent().context("missing image QA directory")?;
        bijux_dna_infra::ensure_dir(qa_dir).context("create image qa dir")?;

        let conn = open_sqlite(&qa_sqlite).context("open image qa sqlite")?;
        ensure_image_qa_tables(&conn).context("ensure image qa tables")?;
        let runner_name = platform.runner.to_string();
        conn.execute(
            "DELETE FROM image_qa_inputs_v1 WHERE platform = ?1 AND runner = ?2",
            (&platform.name, &runner_name),
        )
        .context("reset image qa inputs")?;
        conn.execute(
            "DELETE FROM image_qa_v1 WHERE platform = ?1 AND runner = ?2",
            (&platform.name, &runner_name),
        )
        .context("reset image qa records")?;

        Ok(Self {
            conn,
            qa_jsonl,
            qa_summary: qa_dir.join("qa.json"),
            platform_name: platform.name.clone(),
            runner_name,
        })
    }

    pub(crate) fn conn(&self) -> &rusqlite::Connection {
        &self.conn
    }

    pub(crate) fn insert_input(&self, stage_id: &str, input_hash: &str) -> Result<()> {
        insert_image_qa_input_v1(
            &self.conn,
            stage_id,
            &self.platform_name,
            &self.runner_name,
            input_hash,
        )
        .context("write qa inputs sqlite")
    }

    pub(crate) fn append_record(&self, record: &ImageQaRecord) -> Result<()> {
        append_image_qa_jsonl(&self.qa_jsonl, record).context("write qa jsonl")?;
        insert_image_qa_v1(&self.conn, record).context("write qa sqlite")
    }

    pub(crate) fn write_summary(
        &self,
        pass: usize,
        fail: usize,
        records: &[ImageQaRecord],
    ) -> Result<()> {
        let summary = serde_json::json!({
            "pass": pass,
            "fail": fail,
            "records": records,
        });
        atomic_write_bytes(&self.qa_summary, &serde_json::to_vec_pretty(&summary)?)
            .map_err(anyhow::Error::from)
            .context("write qa.json")
    }
}
