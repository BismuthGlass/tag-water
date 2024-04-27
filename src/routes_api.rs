use rocket::serde::json::Json;
use rocket::State;
use std::path::Path;

use tag_water::commands::{self, models::*};
use tag_water::database::Database;
use tag_water::vault::Vault;

#[post("/tag/new", data = "<input>")]
pub async fn new_tag(db: &State<Database>, input: Json<ReqNewTag>) -> Json<ApiResponse<i64>> {
    Json(commands::new_tag(db, input.into_inner()).await)
}

#[post("/tag/new_many", data = "<input>")]
pub async fn new_tags(db: &State<Database>, input: Json<ReqNewTags>) -> Json<ApiResponse<()>> {
    Json(commands::new_tags(db, input.into_inner()).await)
}

#[post("/tag/find", data = "<input>")]
pub async fn find_tag(
    db: &State<Database>,
    input: Json<ReqFindTag>,
) -> Json<ApiResponse<Vec<String>>> {
    Json(commands::find_tag(db, input.into_inner()).await)
}

#[post("/tag/category/new", data = "<input>")]
pub async fn new_tag_category(
    db: &State<Database>,
    input: Json<ReqNewTagCategory>,
) -> Json<ApiResponse<i64>> {
    Json(commands::new_tag_category(db, input.into_inner()).await)
}

#[post("/tag/category/find", data = "<input>")]
pub async fn find_tag_category(
    db: &State<Database>,
    input: Json<ReqFindTagCategory>,
) -> Json<ApiResponse<Vec<String>>> {
    Json(commands::find_tag_category(db, input.into_inner()).await)
}

#[post("/file/new", data = "<input>")]
pub async fn new_file(
    db: &State<Database>,
    vault: &State<Vault>,
    input: Json<ReqNewFileEntry>,
) -> Json<ApiResponse<i64>> {
    Json(commands::new_file_entry(db, vault, input.into_inner()).await)
}

#[post("/script", data = "<input>")]
pub async fn run_script(
    db: &State<Database>,
    input: Json<RunScriptInput>,
) -> Json<RunScriptOutput> {
    match commands::parse_script(db, &Path::new(&input.work_dir), &Path::new(&input.file)).await {
        Err(logs) => Json(RunScriptOutput {
            status: 400,
            log: Some(logs),
        }),
        Ok(logs) => Json(RunScriptOutput {
            status: 200,
            log: if logs.len() > 0 { Some(logs) } else { None },
        }),
    }
}

#[post("/set/new", data = "<input>")]
pub async fn new_set(db: &State<Database>, input: Json<ReqNewSet>) -> Json<ApiResponse<i64>> {
    Json(commands::new_set(db, input.into_inner()).await)
}
