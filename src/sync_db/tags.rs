use super::{time, Error, Result};
use rusqlite::OptionalExtension;

impl super::SyncDb {
    pub fn verify_tags(&self, tags: &Vec<String>) -> Vec<String> {
        tags.iter()
            .filter(|t| self.count_rows("tag", "name = ?", [t]) == 0)
            .cloned()
            .collect()
    }

    pub fn new_tag_category(&self, name: &str, description: &str) -> Result<i64> {
        if self.count_rows("tag_category", "name = ?", [name]) > 0 {
            return Err(Error::AlreadyExists);
        }

        let mut stmt = self
            .0
            .prepare(
                "insert into tag_category (name, description, time_created, time_updated)
                values (?, ?, ?, ?)",
            )
            .unwrap();
        let time = time();
        Ok(stmt.insert((name, description, time, time)).unwrap())
    }

    pub fn new_tag(&self, name: &str, category: i64, description: &str) -> Result<i64> {
        if self.count_rows("tag", "name = ?", [name]) > 0 {
            return Err(Error::AlreadyExists);
        }

        let mut stmt = self
            .0
            .prepare(
                "insert into tag (name, category, description, time_created, time_updated)
                values (?, ?, ?, ?, ?)",
            )
            .unwrap();
        let time = time();
        Ok(stmt
            .insert((&name, category, &description, time, time))
            .unwrap())
    }

    pub fn tag_category_search(&self, name: &str) -> Vec<String> {
        self.0
            .prepare(
                "select name from tag_category 
                where name like ?
                order by name asc",
            )
            .unwrap()
            .query_map([&format!("%{name}%")], |row| Ok(row.get(0)?))
            .unwrap()
            .map(|v| v.unwrap())
            .collect()
    }

    pub fn tag_search(&self, name: &str, category: Option<i64>) -> Vec<String> {
        let name_search = format!("%{name}%");
        match category {
            Some(c) => self
                .0
                .prepare(
                    "select name from tag where name like ? and category = ? order by name asc",
                )
                .unwrap()
                .query_map((&name_search, c), |row| Ok(row.get(0)?))
                .unwrap()
                .map(|v| v.unwrap())
                .collect(),
            None => self
                .0
                .prepare("select name from tag where name like ? order by name asc")
                .unwrap()
                .query_map([&name_search], |row| Ok(row.get(0)?))
                .unwrap()
                .map(|v| v.unwrap())
                .collect(),
        }
    }

    pub fn tag_id(&self, name: &str) -> Option<i64> {
        self.0
            .query_row("select tag_id from tag where name = ?", [name], |row| {
                row.get(0)
            })
            .optional()
            .unwrap()
    }

    pub fn tag_category_id(&self, name: &str) -> Option<i64> {
        self.0
            .query_row(
                "select tcat_id from tag_category where name = ?",
                [name],
                |row| row.get(0),
            )
            .optional()
            .unwrap()
    }

    pub fn tag_entry(&self, entry_id: i64, tag_id: i64) {
        self.0
            .query_row(
                "select entry_id from entry_tag where entry_id = ? and tag_id = ?",
                [entry_id, tag_id],
                |row| row.get(0),
            )
            .optional()
            .unwrap()
            .and_then(|_: i64| {
                self.0
                    .execute(
                        "insert into entry_tag (entry_id, tag_id) values (?, ?)",
                        [entry_id, tag_id],
                    )
                    .unwrap();
                Some(0)
            });
    }

    pub fn tag_entry_many(&self, entry_id: i64, tag_ids: &[i64]) {
        let mut stmt = self
            .0
            .prepare("select tag_id from entry_tag where entry_id = ?")
            .unwrap();
        let tag_ids: Vec<i64> = stmt
            .query_map((entry_id,), |row| row.get(0))
            .unwrap()
            .map(|row| row.unwrap())
            .filter(|v| !tag_ids.iter().any(|v2| *v2 == *v))
            .collect();

        for tag_id in tag_ids.iter() {
            self.0
                .prepare(
                    "insert into entry_tag (entry_id, tag_id)
                values (?, ?)",
                )
                .unwrap()
                .insert([entry_id, *tag_id])
                .unwrap();
        }
    }
}
