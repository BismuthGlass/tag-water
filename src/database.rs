use rusqlite::OptionalExtension;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use rocket::tokio::task::spawn_blocking;

pub mod models;
pub use models::Error;
use models::Result;

fn time() -> i64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs() as i64
}

fn question_mark_list(number: i64) -> String {
    if number < 1 {
        return "()".to_string();
    }
    let mut string = String::from("(?");
    for _ in 1..number {
        string.push_str(",?");
    }
    string.push(')');
    string
}

pub struct Database(Arc<Mutex<rusqlite::Connection>>);

impl Database {
    pub fn open(file: &Path) -> Self {
        let connection = rusqlite::Connection::open(file).unwrap();

        // Run setup script if the database is new
        let setup = connection
            .query_row("select * from tag_category", [], |_| Ok(()))
            .is_err();
        if setup {
            println!("Setting up database...");
            let db_def = fs::read_to_string("resources/db_def.sql").unwrap();
            connection.execute_batch(&db_def).unwrap();
        }

        connection.execute("PRAGMA foreign_keys = ON", []).unwrap();
        Database(Arc::new(Mutex::new(connection)))
    }

    pub async fn new_tag(&self, name: String, category: i64, description: String) -> Result<i64> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();

            let row: Option<i64> = db
                .query_row("select tag_id from tag where name = ?", [&name], |row| {
                    Ok(row.get(0)?)
                })
                .optional()
                .unwrap();
            if let Some(_) = row {
                return Err(Error::AlreadyExists);
            }

            let mut stmt = db
                .prepare(
                    "insert into tag (name, category, description, time_created, time_updated)
                    values (?, ?, ?, ?, ?)",
                )
                .unwrap();
            let time = time();
            Ok(stmt
                .insert((&name, category, &description, time, time))
                .unwrap())
        })
        .await
        .unwrap()
    }

    pub async fn find_tag(&self, name: String, category: Option<i64>) -> Vec<String> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            let name_search = format!("%{name}%");
            let matches = match category {
                Some(c) => db
                    .prepare(
                        "select name from tag 
                            where name like ? and category = ?
                            order by name asc",
                    )
                    .unwrap()
                    .query_map((name_search, c), |row| Ok(row.get(0)?))
                    .unwrap()
                    .map(|v| v.unwrap())
                    .collect(),
                None => db
                    .prepare(
                        "select name from tag 
                        where name like ?
                        order by name asc",
                    )
                    .unwrap()
                    .query_map([name_search], |row| Ok(row.get(0)?))
                    .unwrap()
                    .map(|v| v.unwrap())
                    .collect(),
            };
            matches
        })
        .await
        .unwrap()
    }

    pub async fn new_tag_category(&self, name: String, description: String) -> Result<i64> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();

            let row: Option<i64> = db
                .query_row(
                    "select tcat_id from tag_category where name = ?",
                    [&name],
                    |row| Ok(row.get(0)?),
                )
                .optional()
                .unwrap();
            if let Some(_) = row {
                return Err(Error::AlreadyExists);
            }

            let mut stmt = db
                .prepare(
                    "insert into tag_category (name, description, time_created, time_updated)
                    values (?, ?, ?, ?)",
                )
                .unwrap();
            let time = time();
            Ok(stmt.insert((&name, &description, time, time)).unwrap())
        })
        .await
        .unwrap()
    }

    pub async fn find_tag_category(&self, name: String) -> Vec<String> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            let name_search = format!("%{name}%");
            let matches = db
                .prepare(
                    "select name from tag_category 
                    where name like ?
                    order by name asc",
                )
                .unwrap()
                .query_map([name_search], |row| Ok(row.get(0)?))
                .unwrap()
                .map(|v| v.unwrap())
                .collect();
            matches
        })
        .await
        .unwrap()
    }

    pub async fn get_tag(&self, name: String) -> Option<i64> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            db.query_row("select tag_id from tag where name = ?", [name], |row| {
                Ok(row.get(0)?)
            })
            .optional()
            .ok()?
        })
        .await
        .unwrap()
    }

    pub async fn get_tag_category(&self, name: String) -> Option<i64> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            db.query_row(
                "select tcat_id from tag_category where name = ?",
                [name],
                |row| Ok(row.get(0)?),
            )
            .optional()
            .unwrap()
        })
        .await
        .unwrap()
    }

    pub async fn verify_tags(&self, tags: Vec<String>) -> Vec<String> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            let mut failures = Vec::new();
            for tag in tags.into_iter() {
                if let None = db
                    .query_row("select tag_id from tag where name = ?", [&tag], |r| {
                        Ok(r.get::<usize, i64>(0))
                    })
                    .optional()
                    .ok()
                {
                    failures.push(tag);
                }
            }
            failures
        })
        .await
        .unwrap()
    }

    pub async fn post_upload(&self, upload: models::UploadFile) -> i64 {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            let mut stmt = db
                .prepare(
                    "insert into upload_file (ext, title)
                    values (?, ?)",
                )
                .unwrap();
            stmt.insert((&upload.ext, &upload.title)).unwrap()
        })
        .await
        .unwrap()
    }

    pub async fn get_upload(&self, id: i64) -> Option<models::UploadFile> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            db.query_row("select * from upload_file where id = ?", [id], |row| {
                Ok(models::UploadFile {
                    id: row.get(0)?,
                    ext: row.get(1)?,
                    title: row.get(2)?,
                })
            })
            .optional()
            .unwrap()
        })
        .await
        .unwrap()
    }

    pub async fn delete_upload(&self, id: i64) {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            db.execute("delete from upload_file where id = ?", [id])
                .unwrap();
        })
        .await
        .unwrap()
    }

    pub async fn intern_upload(&self, upload: models::UploadFile) -> i64 {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            let mut stmt = db
                .prepare(
                    "insert into file (ext, last_update, last_update)
                    values (?, ?, ?)",
                )
                .unwrap();
            stmt.insert((&upload.ext, time(), time())).unwrap()
        })
        .await
        .unwrap()
    }

    pub async fn get_uploads(&self) -> Vec<models::UploadFile> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            let mut stmt = db.prepare("select * from upload_file").unwrap();
            stmt.query_map([], |row| {
                Ok(models::UploadFile {
                    id: row.get(0)?,
                    ext: row.get(1)?,
                    title: row.get(2)?,
                })
            })
            .unwrap()
            .map(|v| v.unwrap())
            .collect()
        })
        .await
        .unwrap()
    }

    pub async fn new_file(&self, ext: String) -> i64 {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            let mut stmt = db
                .prepare(
                    "insert into entry (entry_type, ext, time_created, time_updated)
                    values (1, ?, ?, ?)",
                )
                .unwrap();
            stmt.insert((ext, time(), time())).unwrap()
        })
        .await
        .unwrap()
    }

    // pub async fn query_entries(&self, tags_include: Vec<String>, tags_exclude: Vec<String>) -> Vec<EntryQueryMatch> {
    //     panic!()
    // }

    pub async fn new_set(&self, cover: i64, members: Vec<i64>) -> Result<i64> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();

            for id in &members {
                let parent_set: Option<Option<i64>> = db
                    .query_row(
                        "select parent_set from entry where entry_id = ?",
                        [*id],
                        |row| row.get(0),
                    )
                    .optional()
                    .unwrap();
                if let None = parent_set {
                    return Err(models::Error::InvalidId);
                }
                if let Some(Some(_)) = parent_set {
                    return Err(models::Error::InvalidId);
                }
            }

            let cover_entry_type: Option<i64> = db
                .query_row(
                    "select entry_type from entry where entry_id = ?",
                    [cover],
                    |row| row.get(0),
                )
                .optional()
                .unwrap();
            match cover_entry_type {
                None => return Err(models::Error::InvalidId),
                Some(a) if a != 1 => return Err(models::Error::InvalidId),
                _ => (),
            }

            let mut stmt = db
                .prepare(
                    "insert into entry (entry_type, cover, time_updated, time_created)
                    values (2, ?, ?, ?)",
                )
                .unwrap();
            let time = time();
            let set_id = stmt.insert((cover, time, time)).unwrap();

            let mut data = vec![set_id, time];
            data.append(&mut members.clone());
            db.execute(
                &format!(
                    "update entry set parent_set = ?, time_updated = ? where entry_id in {}",
                    question_mark_list(members.len() as i64)
                ),
                rusqlite::params_from_iter(data),
            )
            .unwrap();

            Ok(set_id)
        })
        .await
        .unwrap()
    }

    pub async fn reorder_set(&self, set_id: i64, entries: Vec<i64>) -> Result<()> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();

            for id in &entries {
                let entry_id: Option<i64> = db
                    .query_row("select entry_id from set_file", [*id], |row| row.get(0))
                    .optional()
                    .unwrap();
                if let None = entry_id {
                    return Err(models::Error::InvalidId);
                }
            }

            for (index, id) in entries.iter().enumerate() {
                db.execute(
                    "update set_file set position = ? where set_id = ? and file_id = ?",
                    [(index + 1) as i64, set_id, *id],
                )
                .unwrap();
            }

            Ok(())
        })
        .await
        .unwrap()
    }

    pub async fn remove_from_set(&self, set_id: i64, files: Vec<i64>) {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();

            for id in files.iter() {
                let f: Option<i64> = db
                    .query_row(
                        "select file_id from set_file where set_id = ? and file_id = ?",
                        [set_id, *id],
                        |row| row.get(0),
                    )
                    .optional()
                    .unwrap();
                if let None = f {
                    continue;
                }

                db.execute(
                    "delete from set_file where set_id = ? and file_id = ?",
                    [set_id, *id],
                )
                .unwrap();
                db.execute(
                    "update entry set parent_set = null where entry_id = ?",
                    [id],
                )
                .unwrap();
            }
        })
        .await
        .unwrap()
    }

    pub async fn add_to_set(&self, set_id: i64, files: Vec<i64>) -> Result<()> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();

            // Check if all files are valid
            for id in files.iter() {
                let entry_info: Option<(i64, Option<i64>)> = db
                    .query_row(
                        "select entry_type from entry where entry_id = ?",
                        [*id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()
                    .unwrap();

                if let Some((entry_type, parent_set)) = entry_info {
                    if entry_type != 1 {
                        return Err(models::Error::InvalidId);
                    }
                    if let Some(_) = parent_set {
                        return Err(models::Error::InvalidId);
                    }
                } else {
                    return Err(models::Error::NotFound);
                }
            }

            for id in files.iter() {
                db.execute(
                    "insert into set_file (set_id, file_id, position)
                    select ?, ?, max(position) + 1 
                    from set_file where set_id = ?",
                    [set_id, *id, set_id],
                )
                .unwrap();
            }
            Ok(())
        })
        .await
        .unwrap()
    }

    pub async fn set_set_cover(&self, set_id: i64, cover: i64) -> Result<()> {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();

            // Check if cover is valid
            let cover_entry_type: Option<i64> = db
                .query_row(
                    "select type from entry where entry_id = ?",
                    [cover],
                    |row| row.get(0),
                )
                .optional()
                .unwrap();
            match cover_entry_type {
                None => return Err(models::Error::NotFound),
                Some(a) if a != 1 => return Err(models::Error::InvalidCover),
                _ => (),
            }

            db.execute(
                "update entry set cover = ? where entry_id = ?",
                [cover, set_id],
            )
            .unwrap();
            Ok(())
        })
        .await
        .unwrap()
    }

    pub async fn add_entry_tag(&self, entry_id: i64, tag_id: i64) {
        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            db.query_row(
                "select entry_id from entry_tag where entry_id = ? and tag_id = ?",
                [entry_id, tag_id],
                |row| row.get(0),
            )
            .optional()
            .unwrap()
            .and_then(|_: i64| {
                let mut stmt = db
                    .prepare(
                        "insert into entry_tag (entry_id, tag_id)
                        values (?, ?)",
                    )
                    .unwrap();
                stmt.insert([entry_id, tag_id]).unwrap();
                Some(0)
            });
        })
        .await
        .unwrap();
    }

    pub async fn add_entry_tag_many(&self, entry_id: i64, tag_ids: &[i64]) {
        let t_db = Arc::clone(&self.0);
        let tag_ids = Vec::from(tag_ids);
        spawn_blocking(move || {
            let db = t_db.lock().unwrap();
            let mut stmt = db
                .prepare("select tag_id from entry_tag where entry_id = ?")
                .unwrap();
            let tag_ids: Vec<i64> = stmt
                .query_map((entry_id,), |row| row.get(0))
                .unwrap()
                .map(|row| row.unwrap())
                .filter(|v| !tag_ids.iter().any(|v2| *v2 == *v))
                .collect();

            for tag_id in tag_ids.iter() {
                db.prepare(
                    "insert into entry_tag (entry_id, tag_id)
                    values (?, ?)",
                )
                .unwrap()
                .insert([entry_id, *tag_id])
                .unwrap();
            }
        })
        .await
        .unwrap();
    }

    pub async fn query(
        &self,
        query_info: &models::EntryQuery,
        page: i64,
        page_size: i64,
    ) -> models::EntryQueryResult {
        let (conditions, args) = query_info.generate_query();
        let page_offset = (page - 1) * page_size;

        let t_db = Arc::clone(&self.0);
        spawn_blocking(move || {
            let count_query = format!("select count(*) from entry e where {conditions}");
            let result_query = format!(
                "select 
                    e.entry_id, 
                    e.entry_type,
                    e.parent_set,
                    e.ext, 
                    ec.entry_id, 
                    ec.ext
                from entry e
                left join entry ec on e.cover = ec.entry_id
                where {conditions}
                limit {page_size} offset {page_offset}"
            );

            println!("{result_query}");

            let db = t_db.lock().unwrap();

            let entry_count: i64 = db
                .query_row(&count_query, rusqlite::params_from_iter(&args), |r| {
                    r.get(0)
                })
                .unwrap();

            let mut entries = Vec::new();
            let mut stmt = db.prepare(&result_query).unwrap();
            let mut rows = stmt.query(rusqlite::params_from_iter(&args)).unwrap();
            while let Some(r) = rows.next().unwrap() {
                let id = r.get(0).unwrap();
                let kind = r.get(1).unwrap();
                let parent_set = r.get(2).ok();
                let (img_id, img_ext) = match kind {
                    // Dealing with a file
                    1 => (id, r.get(3).expect("{id} has no ext but is file")),
                    // Dealing with a set
                    2 => (
                        r.get(4).expect("{id} is set with no cover"),
                        r.get(5).expect("{id} is set with no cover"),
                    ),
                    _ => panic!("Wrong type for entry {id}"),
                };
                entries.push(models::EntryQueryMatch {
                    id,
                    kind,
                    parent_set,
                    img_id,
                    img_ext,
                });
            }

            // Calculate pagination info
            let page_count = (entry_count - 1) / page_size + 1;

            models::EntryQueryResult {
                entry_count,
                page: page,
                page_size: page_size,
                page_count: page_count,
                page_entries: entries,
            }
        })
        .await
        .unwrap()
    }
}
