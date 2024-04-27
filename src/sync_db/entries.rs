use super::question_mark_list;
use super::{time, Error, Result};
use rusqlite::OptionalExtension;

impl super::SyncDb {
    pub fn new_file(&self, ext: String) -> i64 {
        let mut stmt = self
            .0
            .prepare(
                "insert into entry (entry_type, ext, time_created, time_updated)
                values (1, ?, ?, ?)",
            )
            .unwrap();
        stmt.insert((ext, time(), time())).unwrap()
    }

    pub fn new_set(&self, cover: i64, members: Vec<i64>) -> Result<i64> {
        for id in &members {
            let parent_set: Option<Option<i64>> = self
                .0
                .query_row(
                    "select parent_set from entry where entry_id = ?",
                    [*id],
                    |row| row.get(0),
                )
                .optional()
                .unwrap();
            if let None = parent_set {
                return Err(Error::NotFound);
            }
            if let Some(Some(_)) = parent_set {
                return Err(Error::InvalidId);
            }
        }

        let cover_entry_type: Option<i64> = self
            .0
            .query_row(
                "select entry_type from entry where entry_id = ?",
                [cover],
                |row| row.get(0),
            )
            .optional()
            .unwrap();
        match cover_entry_type {
            None => return Err(Error::InvalidId),
            Some(a) if a != 1 => return Err(Error::InvalidId),
            _ => (),
        }

        let mut stmt = self
            .0
            .prepare(
                "insert into entry (entry_type, cover, time_updated, time_created)
                values (2, ?, ?, ?)",
            )
            .unwrap();
        let time = time();
        let set_id = stmt.insert((cover, time, time)).unwrap();

        let mut data = vec![set_id, time];
        data.append(&mut members.clone());
        self.0
            .execute(
                &format!(
                    "update entry set parent_set = ?, time_updated = ? where entry_id in {}",
                    question_mark_list(members.len() as i64)
                ),
                rusqlite::params_from_iter(data),
            )
            .unwrap();

        Ok(set_id)
    }

    pub fn reorder_set(&self, set_id: i64, entries: Vec<i64>) -> Result<()> {
        for id in &entries {
            let entry_id: Option<i64> = self
                .0
                .query_row("select entry_id from set_file", [*id], |row| row.get(0))
                .optional()
                .unwrap();
            if let None = entry_id {
                return Err(Error::InvalidId);
            }
        }

        for (index, id) in entries.iter().enumerate() {
            self.0
                .execute(
                    "update set_file set position = ? where set_id = ? and file_id = ?",
                    [(index + 1) as i64, set_id, *id],
                )
                .unwrap();
        }

        Ok(())
    }

    pub async fn remove_from_set(&self, set_id: i64, files: Vec<i64>) {
        for id in files.iter() {
            if self.count_rows("set_file", "set_id = ? and file_id = ?", [set_id, *id]) == 0 {
                continue;
            }

            self.0
                .execute(
                    "delete from set_file where set_id = ? and file_id = ?",
                    [set_id, *id],
                )
                .unwrap();
            self.0
                .execute(
                    "update entry set parent_set = null where entry_id = ?",
                    [id],
                )
                .unwrap();
        }
    }

    pub fn add_to_set(&self, set_id: i64, files: Vec<i64>) -> Result<()> {
        // Check if all files are valid
        for id in files.iter() {
            let entry_info: Option<(i64, Option<i64>)> = self
                .0
                .query_row(
                    "select entry_type from entry where entry_id = ?",
                    [*id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .unwrap();

            if let Some((entry_type, parent_set)) = entry_info {
                if entry_type != 1 {
                    return Err(Error::InvalidId);
                }
                if let Some(_) = parent_set {
                    return Err(Error::InvalidId);
                }
            } else {
                return Err(Error::NotFound);
            }
        }

        for id in files.iter() {
            self.0
                .execute(
                    "insert into set_file (set_id, file_id, position)
                select ?, ?, max(position) + 1 
                from set_file where set_id = ?",
                    [set_id, *id, set_id],
                )
                .unwrap();
        }
        Ok(())
    }

    pub fn set_cover(&self, set_id: i64, cover: i64) -> Result<()> {
        // Check if cover is valid
        let cover_entry_type: Option<i64> = self
            .0
            .query_row(
                "select type from entry where entry_id = ?",
                [cover],
                |row| row.get(0),
            )
            .optional()
            .unwrap();
        match cover_entry_type {
            None => return Err(Error::NotFound),
            Some(a) if a != 1 => return Err(Error::InvalidCover),
            _ => (),
        }

        self.0
            .execute(
                "update entry set cover = ? where entry_id = ?",
                [cover, set_id],
            )
            .unwrap();
        Ok(())
    }
}
