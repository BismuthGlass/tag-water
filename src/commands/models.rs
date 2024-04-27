use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub status: i64,
    pub messages: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        return Self {
            status: 200,
            messages: Vec::new(),
            data: Some(data),
        };
    }

    pub fn err(messages: Vec<String>) -> Self {
        return Self {
            status: 400,
            messages,
            data: None,
        };
    }

    pub fn ok_plus(messages: Vec<String>, data: T) -> Self {
        return Self {
            status: 200,
            messages,
            data: Some(data),
        };
    }
}

#[derive(Deserialize)]
pub struct RunScriptInput {
    pub work_dir: String,
    pub file: String,
}

#[derive(Deserialize)]
pub struct ReqNewTag {
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct ReqNewTagCategory {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct ReqNewFileEntry {
    pub file: String,
    pub tags: Vec<String>,
}

#[derive(Serialize)]
pub struct RunScriptOutput {
    pub status: i64,
    pub log: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct ReqFindTag {
    pub name: String,
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct ReqFindTagCategory {
    pub name: String,
}

#[derive(Deserialize)]
pub struct ReqNewTags {
    pub tags: Vec<String>,
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct ReqNewSet {
    pub cover: Option<i64>,
    pub files: Vec<i64>,
}

#[derive(Deserialize)]
pub struct ReqAddToSet {
    pub set_id: i64,
    // (id, position)
    pub files: Vec<(i64, i64)>,
}

#[derive(Deserialize)]
pub struct ReqRemoveFromSet {
    pub set_id: i64,
    pub files: Vec<i64>,
}

#[derive(Deserialize)]
pub struct ReqChangeSet {
    pub set_id: i64,
    pub file_order: Vec<i64>,
}
