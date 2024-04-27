pub mod models;
mod script_parser;

use crate::database::{self, Database};
use crate::vault::Vault;
pub use models::*;
use rocket::tokio::fs;
use rocket::tokio::io::AsyncReadExt;
use rocket::State;
use std::collections::HashMap;
use std::path::Path;

fn extension(file: &Path) -> String {
    file.extension()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_str()
        .unwrap()
        .to_string()
}

pub async fn parse_script(
    db: &State<Database>,
    work_dir: &Path,
    file: &Path,
) -> Result<Vec<String>, Vec<String>> {
    let script_path = work_dir.join(file);
    let file = rocket::tokio::fs::File::open(&script_path).await;
    if let Err(e) = &file {
        if e.kind() == std::io::ErrorKind::NotFound {
            return Err(vec![format!("File {file:?} not found")]);
        }
    }
    let mut file = file.unwrap();
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents).await.unwrap();
    let script_data = (|| {
        let tokens = script_parser::tokenizer::tokenize(file_contents.bytes())?;
        let script_data = script_parser::Data::from_tokens(&tokens)?;
        Ok(script_data)
    })();
    if let Err(s) = script_data {
        return Err(vec![s]);
    }
    let script_data = script_data.unwrap();

    // Verify tags
    let tags: Vec<String> = script_data.tags.iter().cloned().collect();
    let failures = db.verify_tags(tags).await;
    if failures.len() > 0 {
        return Err(vec![
            "Unknown tags:".to_string(),
            format!("\t{}", failures.join(", ")),
        ]);
    }

    let mut tag_dict = HashMap::new();
    for tag in &script_data.tags {
        let tag_id = db.get_tag(tag.clone()).await.unwrap();
        tag_dict.insert(tag.clone(), tag_id);
    }

    // Verify if all files exist
    let mut file_errors = Vec::new();
    for file in &script_data.files {
        let path = Path::new("data/files").join(&file.file);
        if let Err(_) = fs::metadata(&path).await {
            file_errors.push(file.file.clone());
        }
    }
    if file_errors.len() > 0 {
        return Err(vec![
            "Could not read files:".to_string(),
            format!("\t'{}'", file_errors.join(", ")),
        ]);
    }

    // Script parser output goes here
    let mut logs = Vec::new();

    let mut file_ids = Vec::new();
    for file in &script_data.files {
        // Copy files
        let ext = extension(Path::new(&file.file));
        let new_file_id = db.new_file(ext.clone()).await;
        file_ids.push(new_file_id);

        // Tag new entry
        let mut tag_list = Vec::new();
        for tag in &file.tags {
            tag_list.push(*tag_dict.get(tag.as_str()).unwrap());
        }
        db.add_entry_tag_many(new_file_id, &tag_list).await;

        let file_path = work_dir.join(&file.file);
        let dst_file_name = format!("{new_file_id}.{ext}");
        let dst_path = Path::new("data/files").join(&dst_file_name);
        if let Err(e) = fs::copy(file_path, dst_path).await {
            logs.push(format!("Error copying '{}': {e}", file.file));
        }
    }

    // Create sets
    for set in &script_data.sets {
        let new_entry = db
            .new_set(file_ids[set.files[0]], file_ids.clone())
            .await
            .unwrap();

        let mut tag_list: Vec<i64> = Vec::new();
        for tag in &set.tags {
            tag_list.push(*tag_dict.get(tag.as_str()).unwrap());
        }
        db.add_entry_tag_many(new_entry, &tag_list).await;
    }

    Ok(logs)
}

pub async fn new_tag(db: &State<Database>, input: ReqNewTag) -> ApiResponse<i64> {
    let category = match input.category {
        Some(v) => match db.get_tag_category(v.clone()).await {
            Some(v) => v,
            None => {
                return ApiResponse {
                    status: 400,
                    messages: vec![format!("Unknown category '{v}'")],
                    data: None,
                }
            }
        },
        None => 1, // default group
    };

    let id = db
        .new_tag(
            input.name.clone(),
            category,
            input.description.unwrap_or("".to_string()),
        )
        .await;
    let id = match id {
        Ok(v) => v,
        Err(_) => {
            return ApiResponse::err(vec![format!("Tag '{}' already exists", input.name)]);
        }
    };

    ApiResponse::ok(id)
}

pub async fn new_tags(db: &State<Database>, input: ReqNewTags) -> ApiResponse<()> {
    let category = input.category.unwrap_or("default".to_string());
    let category = match db.get_tag_category(category.clone()).await {
        None => return ApiResponse::err(vec![format!("Tag category {category} does not exist")]),
        Some(id) => id,
    };

    let mut log = Vec::new();
    for t in &input.tags {
        match db.new_tag(t.clone(), category, "".to_string()).await {
            Err(_) => log.push(format!("Tag {t} already exists")),
            Ok(_) => (),
        }
    }

    let res = ApiResponse::ok_plus(log, ());
    println!("{}", serde_json::to_string(&res).unwrap());
    res
}

pub async fn find_tag(db: &State<Database>, input: ReqFindTag) -> ApiResponse<Vec<String>> {
    let category = match &input.category {
        None => None,
        Some(c) => {
            let id = db.get_tag_category(c.clone()).await;
            if let None = id {
                return ApiResponse::err(vec![format!(
                    "Category '{}' not found",
                    input.category.unwrap()
                )]);
            }
            id
        }
    };

    ApiResponse::ok(db.find_tag(input.name, category).await)
}

pub async fn find_tag_category(
    db: &State<Database>,
    input: ReqFindTagCategory,
) -> ApiResponse<Vec<String>> {
    ApiResponse::ok(db.find_tag_category(input.name).await)
}

pub async fn new_tag_category(db: &State<Database>, input: ReqNewTagCategory) -> ApiResponse<i64> {
    let id = match db
        .new_tag_category(
            input.name.clone(),
            input.description.unwrap_or("".to_string()),
        )
        .await
    {
        Ok(v) => v,
        Err(_) => {
            return ApiResponse::err(vec![format!("Tag group '{}' already exists", input.name)])
        }
    };

    ApiResponse::ok(id)
}

pub async fn new_file_entry(
    db: &State<Database>,
    vault: &State<Vault>,
    input: ReqNewFileEntry,
) -> ApiResponse<i64> {
    let mut tag_ids = Vec::new();
    let mut unknown_tags = Vec::new();
    for tag in &input.tags {
        match db.get_tag(tag.clone()).await {
            None => unknown_tags.push(tag.clone()),
            Some(id) => tag_ids.push(id),
        }
    }
    if unknown_tags.len() > 1 {
        return ApiResponse::err(vec![format!("Unknown tags: {}", unknown_tags.join(" "))]);
    }

    let file = Path::new(&input.file);

    if let Err(_) = fs::metadata(&file).await {
        return ApiResponse::err(vec!["Could not read file '{file}'".to_string()]);
    }

    let ext = file
        .extension()
        .map(|e| e.to_str().unwrap())
        .unwrap_or("")
        .to_string();

    let id = db.new_file(ext).await;
    db.add_entry_tag_many(id, &tag_ids).await;
    vault.intern_file(file, id).await;

    ApiResponse::ok(id)
}

pub async fn new_set(db: &State<Database>, input: ReqNewSet) -> ApiResponse<i64> {
    if input.files.len() == 0 {
        return ApiResponse::err(vec!["Cannot create empty set".to_string()]);
    }
    let cover = if let Some(id) = input.cover {
        id
    } else {
        input.files[0]
    };
    match db.new_set(cover, input.files).await {
        Ok(id) => return ApiResponse::ok(id),
        Err(database::models::Error::InvalidCover) => {
            return ApiResponse::err(vec!["Invalid cover".to_string()]);
        }
        Err(database::models::Error::InvalidId) => {
            return ApiResponse::err(vec!["Invalid file id".to_string()]);
        }
        Err(e) => {
            return ApiResponse::err(vec![format!("Unknown error {e:?}")]);
        }
    }
}
