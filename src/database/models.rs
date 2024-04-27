use rusqlite::types::{ToSql, ToSqlOutput};
use serde::Serialize;

type Timestamp = i64;

#[derive(Debug)]
pub enum Error {
    InvalidId,
    AlreadyExists,
    NotFound,
    InvalidCover,
    BelongsToSet,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Tag {
    pub id: i64,
    pub name: String,
    pub category: i64,
    pub description: Option<String>,
    pub last_update: i64,
    pub time_created: i64,
}

pub struct File {
    pub id: i64,
    pub ext: String,
    pub last_update: i64,
    pub time_created: i64,
}

pub struct Set {
    pub id: i64,
    pub cover: i64,
    pub time_created: i64,
    pub last_update: i64,
}

pub struct UploadFile {
    pub id: i64,
    pub ext: String,
    pub title: String,
}

#[derive(Serialize)]
pub struct EntryQueryMatch {
    pub kind: i64,
    pub id: i64,
    pub parent_set: Option<i64>,
    pub img_id: i64,
    pub img_ext: String,
}

#[derive(Serialize)]
pub struct EntryQueryResult {
    pub entry_count: i64,
    pub page: i64,
    pub page_size: i64,
    pub page_count: i64,
    pub page_entries: Vec<EntryQueryMatch>,
}

#[derive(Default, Debug)]
pub struct EntryQuery {
    pub tags_included: Vec<i64>,
    pub tags_excluded: Vec<i64>,
    pub created_after: Option<Timestamp>,
    pub created_before: Option<Timestamp>,
    pub updated_after: Option<Timestamp>,
    pub updated_before: Option<Timestamp>,
    pub is_set: bool,
    pub is_file: bool,
    pub untagged: bool,
    pub include_set_files: bool,
}

pub enum EntryQueryParam {
    String(String),
    Int(i64),
}

impl ToSql for EntryQueryParam {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            EntryQueryParam::String(s) => s.to_sql(),
            EntryQueryParam::Int(i) => i.to_sql(),
        }
    }
}

impl EntryQuery {
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

    pub fn generate_query(&self) -> (String, Vec<EntryQueryParam>) {
        let mut query_parts = Vec::new();
        let mut params = Vec::new();
        if self.tags_included.len() > 0 {
            for id in &self.tags_included {
                params.push(EntryQueryParam::Int(*id));
            }
            query_parts.push(format!(
                "exists (
                        select 1
                        from entry_tag et
                        where et.entry_id = e.entry_id
                        and et.tag_id in {}
                        group by et.entry_id
                        having count(distict et.tag_id) = 3
                    )",
                Self::question_mark_list(self.tags_included.len() as i64)
            ));
        }

        if self.tags_excluded.len() > 0 {
            for id in &self.tags_excluded {
                params.push(EntryQueryParam::Int(*id));
            }
            query_parts.push(format!(
                "not exists (
                    select 1
                    from entry_tag et
                    where et.entry_id = e.entry_id
                    and et.tag_id in {}
                )",
                Self::question_mark_list(self.tags_excluded.len() as i64),
            ));
        }

        if let Some(timestamp) = self.created_after {
            params.push(EntryQueryParam::Int(timestamp));
            query_parts.push("e.time_created > ?".to_string())
        }
        if let Some(timestamp) = self.created_before {
            params.push(EntryQueryParam::Int(timestamp));
            query_parts.push("e.time_created < ?".to_string())
        }
        if let Some(timestamp) = self.updated_after {
            params.push(EntryQueryParam::Int(timestamp));
            query_parts.push("e.time_updated > ?".to_string())
        }
        if let Some(timestamp) = self.updated_before {
            params.push(EntryQueryParam::Int(timestamp));
            query_parts.push("e.time_updated < ?".to_string())
        }

        if self.is_set {
            query_parts.push("e.entry_type = 2".to_string());
        }
        if self.is_file {
            query_parts.push("e.entry_type = 1".to_string());
        }

        if self.untagged {
            query_parts.push(
                "not exists (
                        select 1 from entry_tag et
                        where et.entry_id = e.entry_id
                )"
                .to_string(),
            );
        }
        if !self.include_set_files {
            query_parts.push("e.parent_set is null".to_string())
        }

        (query_parts.join(" and "), params)
    }
}
