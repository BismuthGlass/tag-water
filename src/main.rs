#![allow(dead_code)]

#[macro_use]
extern crate rocket;

mod routes_api;
mod routes_web;

use std::path::{Path, PathBuf};

use rocket::http::ContentType;
use rocket::tokio::fs::File;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use tag_water::vault::Vault;

async fn retrieve_file(file: &Path) -> Option<(ContentType, File)> {
    let ctype = match file.extension() {
        Some(v) => ContentType::from_extension(v.to_str().unwrap()).unwrap_or(ContentType::Plain),
        None => ContentType::Plain,
    };
    let file = File::open(file).await.ok()?;
    Some((ctype, file))
}

#[get("/")]
async fn index() -> Template {
    Template::render("pages/index", context! {})
}

#[get("/static/<file..>")]
async fn static_file(file: PathBuf) -> Option<(ContentType, File)> {
    retrieve_file(&Path::new("Resources").join(file)).await
}

#[get("/thumb/<id>")]
async fn thumb(vault: &State<Vault>, id: i64) -> Option<(ContentType, File)> {
    let file = format!("{}.jpg", id);
    retrieve_file(&vault.storage_thumb_dir.join(file)).await
}

#[get("/upload/thumb/<id>")]
async fn upload_thumb(vault: &State<Vault>, id: i64) -> Option<(ContentType, File)> {
    let file = format!("{}.jpg", id);
    retrieve_file(&vault.upload_thumb_dir.join(file)).await
}

#[launch]
fn rocket() -> _ {
    let vault_location = Path::new("./data");
    rocket::build()
        .mount("/", routes![index, static_file, thumb, upload_thumb])
        .mount(
            "/",
            routes![
                routes_web::page_upload,
                routes_web::page_gallery,
                routes_web::page_tags,
                routes_web::post_upload,
                routes_web::delete_upload,
            ],
        )
        .mount(
            "/api",
            routes![
                routes_api::new_tag,
                routes_api::new_tags,
                routes_api::find_tag,
                routes_api::new_tag_category,
                routes_api::find_tag_category,
                routes_api::new_file,
                routes_api::new_set,
            ],
        )
        .attach(Template::fairing())
        .manage(tag_water::database::Database::open(
            &vault_location.join("db.sqlite"),
        ))
        .manage(tag_water::vault::Vault::open(&vault_location))
}
