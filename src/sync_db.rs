//use std::fs;
use rusqlite::Params;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod entries;
pub mod models;
pub mod tags;
pub use models::Error;
pub use models::Result;

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

pub struct SyncDb(rusqlite::Connection);

impl SyncDb {
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
        Self(connection)
    }

    fn count_rows(&self, table: &str, conditions: &str, params: impl Params) -> i64 {
        let query = format!("select * from {table} where {conditions}");
        self.0.query_row(&query, params, |row| row.get(0)).unwrap()
    }
}
