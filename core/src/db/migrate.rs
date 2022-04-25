use crate::prisma::{self, migration};
use anyhow::Result;
use data_encoding::HEXLOWER;
use include_dir::{include_dir, Dir};
use ring::digest::{Context, Digest, SHA256};
use std::ffi::OsStr;
use std::io::{BufReader, Read};

const INIT_MIGRATION: &str = include_str!("../../prisma/migrations/migration_table/migration.sql");
static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/prisma/migrations");

pub fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest> {
  let mut context = Context::new(&SHA256);
  let mut buffer = [0; 1024];
  loop {
    let count = reader.read(&mut buffer)?;
    if count == 0 {
      break;
    }
    context.update(&buffer[..count]);
  }
  Ok(context.finish())
}

pub async fn run_migrations(db_url: &str) -> Result<()> {
  let client = prisma::new_client_with_url(&format!("file:{}", &db_url)).await?;

  match client
    ._query_raw::<serde_json::Value>(
      "SELECT name FROM sqlite_master WHERE type='table' AND name='_migrations'",
    )
    .await
  {
    Ok(data) => {
      if data.len() == 0 {
        #[cfg(debug_assertions)]
        println!("Migration table does not exist");
        // execute migration
        match client._execute_raw(INIT_MIGRATION).await {
          Ok(_) => {}
          Err(e) => {
            println!("Failed to create migration table: {}", e);
          }
        };

        let value: Vec<serde_json::Value> = client
          ._query_raw("SELECT name FROM sqlite_master WHERE type='table' AND name='_migrations'")
          .await
          .unwrap();

        #[cfg(debug_assertions)]
        println!("Migration table created: {:?}", value);
      } else {
        #[cfg(debug_assertions)]
        println!("Migration table exists: {:?}", data);
      }

      let mut migration_subdirs = MIGRATIONS_DIR
        .dirs()
        .filter(|subdir| {
          subdir
            .path()
            .file_name()
            .map(|name| name != OsStr::new("migration_table"))
            .unwrap_or(false)
        })
        .collect::<Vec<_>>();

      migration_subdirs.sort_by(|a, b| {
        let a_name = a.path().file_name().unwrap().to_str().unwrap();
        let b_name = b.path().file_name().unwrap().to_str().unwrap();

        let a_time = a_name[..14].parse::<i64>().unwrap();
        let b_time = b_name[..14].parse::<i64>().unwrap();

        a_time.cmp(&b_time)
      });

      for subdir in migration_subdirs {
        println!("{:?}", subdir.path());
        let migration_file = subdir
          .get_file(subdir.path().join("./migration.sql"))
          .unwrap();
        let migration_sql = migration_file.contents_utf8().unwrap();

        let digest = sha256_digest(BufReader::new(migration_file.contents()))?;
        // create a lowercase hash from
        let checksum = HEXLOWER.encode(digest.as_ref());
        let name = subdir.path().file_name().unwrap().to_str().unwrap();

        // get existing migration by checksum, if it doesn't exist run the migration
        let existing_migration = client
          .migration()
          .find_unique(migration::checksum::equals(checksum.clone()))
          .exec()
          .await?;

        if existing_migration.is_none() {
          #[cfg(debug_assertions)]
          println!("Running migration: {}", name);

          let steps = migration_sql.split(";").collect::<Vec<&str>>();
          let steps = &steps[0..steps.len() - 1];

          client
            .migration()
            .create(
              migration::name::set(name.to_string()),
              migration::checksum::set(checksum.clone()),
              vec![],
            )
            .exec()
            .await?;

          for (i, step) in steps.iter().enumerate() {
            match client._execute_raw(&format!("{};", step)).await {
              Ok(_) => {
                #[cfg(debug_assertions)]
                println!("Step {} ran successfully", i);
                client
                  .migration()
                  .find_unique(migration::checksum::equals(checksum.clone()))
                  .update(vec![migration::steps_applied::set(i as i32 + 1)])
                  .exec()
                  .await?;
              }
              Err(e) => {
                println!("Error running migration: {}", name);
                println!("{}", e);
                break;
              }
            }
          }

          #[cfg(debug_assertions)]
          println!("Migration {} recorded successfully", name);
        } else {
          #[cfg(debug_assertions)]
          println!("Migration {} already exists", name);
        }
      }
    }
    Err(err) => {
      panic!("Failed to check migration table existence: {:?}", err);
    }
  }

  Ok(())
}
