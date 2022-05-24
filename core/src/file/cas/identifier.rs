use crate::job::jobs::JobReportUpdate;
use crate::{
	file::FileError,
	job::{jobs::Job, worker::WorkerContext},
	prisma::file_path,
	CoreContext,
};
use anyhow::Result;
use futures::executor::block_on;
use prisma_client_rust::Direction;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct FileCreated {
	pub id: i32,
	pub cas_id: String,
}

#[derive(Debug)]
pub struct FileIdentifierJob;

#[async_trait::async_trait]
impl Job for FileIdentifierJob {
	async fn run(&self, ctx: WorkerContext) -> Result<()> {
		let total_count = count_orphan_file_paths(&ctx.core_ctx).await?;
		println!("Found {} orphan file paths", total_count);

		let task_count = (total_count as f64 / 100f64).ceil() as usize;

		println!("Will process {} tasks", task_count);

		// update job with total task count based on orphan file_paths count
		ctx.progress(vec![JobReportUpdate::TaskCount(task_count)]);

		let db = ctx.core_ctx.database.clone();

		let ctx = tokio::task::spawn_blocking(move || {
      let mut completed: usize = 0;
      let mut cursor: i32 = 1;

      while completed < task_count {

        let file_paths = block_on(get_orphan_file_paths(&ctx.core_ctx, cursor)).unwrap();
        println!("Processing {:?} orphan files. ({} completed of {})", file_paths.len(), completed, task_count);

        let mut rows: Vec<String> = Vec::new();
        // only rows that have a valid cas_id to be inserted
        for file_path in file_paths.iter() {
          if file_path.temp_cas_id.is_some() {
            rows.push(prepare_file_values(file_path));
          }
        }
        if rows.len() == 0 {
          println!("No orphan files to process, finishing...");
          break;
        }
        let insert_files = format!(
          r#"INSERT INTO files (cas_id, size_in_bytes) VALUES {} ON CONFLICT (cas_id) DO NOTHING RETURNING id, cas_id"#,
          rows.join(", ")
        );
        
        let files: Vec<FileCreated> = block_on(db._query_raw(&insert_files)).unwrap();

        for file in files.iter() {
          let update_file_path = format!(
            r#"UPDATE file_paths SET file_id = "{}" WHERE temp_cas_id = "{}""#,
            file.id, file.cas_id
          );
          block_on(db._execute_raw(&update_file_path)).unwrap();
        }

        let last_row = file_paths.last().unwrap();

        cursor = last_row.id;
        
        completed += 1;
        ctx.progress(vec![
          JobReportUpdate::CompletedTaskCount(completed),
          JobReportUpdate::Message(format!(
            "Processed {} of {} orphan files",
            completed,
            task_count
          )),
        ]);
      }
      ctx
    }).await?;

		let remaining = count_orphan_file_paths(&ctx.core_ctx).await?;

		println!(
			"Finished with {} files remaining because your code is bad.",
			remaining
		);

		// if remaining > 0 {
		//   ctx.core_ctx.spawn_job(Box::new(FileIdentifierJob));
		// }

		Ok(())
	}
}

#[derive(Deserialize, Serialize, Debug)]
struct CountRes {
	count: Option<usize>,
}

pub async fn count_orphan_file_paths(ctx: &CoreContext) -> Result<usize, FileError> {
	let db = &ctx.database;
	let files_count = db
		._query_raw::<CountRes>(
			r#"SELECT COUNT(*) AS count FROM file_paths WHERE file_id IS NULL AND is_dir IS FALSE"#,
		)
		.await?;
	Ok(files_count[0].count.unwrap_or(0))
}

pub async fn get_orphan_file_paths(
	ctx: &CoreContext,
	cursor: i32,
) -> Result<Vec<file_path::Data>, FileError> {
	let db = &ctx.database;
	println!("cursor: {:?}", cursor);
	let files = db
		.file_path()
		.find_many(vec![
			file_path::file_id::equals(None),
			file_path::is_dir::equals(false),
		])
		.order_by(file_path::id::order(Direction::Asc))
		.cursor(file_path::id::cursor(cursor))
		.take(100)
		.exec()
		.await?;
	Ok(files)
}

pub fn prepare_file_values(file_path: &file_path::Data) -> String {
	format!(
		"(\"{}\",\"{}\")",
		file_path.temp_cas_id.as_ref().unwrap(),
		"0"
	)
}
