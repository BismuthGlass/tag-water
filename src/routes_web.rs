use rocket::form::Form;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use tag_water::database::Database;

mod models;
use models::*;

#[get("/upload")]
pub async fn page_upload(db: &State<Database>) -> Template {
    let uploads: Vec<UploadFile> = db
        .get_uploads()
        .await
        .into_iter()
        .map(UploadFile::from_model)
        .collect();
    Template::render("pages/upload", context! { uploads: uploads })
}

#[get("/gallery?<query>&<page>&<page_size>")]
pub async fn page_gallery(
    db: &State<Database>,
    query: Option<&str>,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Template {
    let keep_query = query.map(|q| q.to_string());
    let query = query
        .map(|v| v.split(" ").map(|v| v.to_string()).collect())
        .unwrap_or(Vec::new());
    let query = match tag_water::query::parse_query_string(db, &query).await {
        Ok(v) => v,
        Err(msg) => {
            return Template::render(
                "pages/gallery",
                &GalleryCtx {
                    error: Some(msg),
                    query: keep_query,
                    data: None,
                },
            )
        }
    };

    Template::render(
        "pages/gallery",
        &GalleryCtx {
            error: None,
            query: keep_query,
            data: Some(
                db.query(&query, page.unwrap_or(1), page_size.unwrap_or(50))
                    .await,
            ),
        },
    )
}

#[get("/tags")]
pub async fn page_tags(_db: &State<Database>) -> Template {
    Template::render("pages/tags", context! {})
}

#[post("/upload", data = "<data>")]
pub async fn post_upload(db: &State<Database>, mut data: Form<UploadFileForm<'_>>) -> Template {
    let uploads = data.process(db).await;
    Template::render("components/upload_cards", context! { uploads: uploads })
}

#[delete("/upload/<id>")]
pub async fn delete_upload(db: &State<Database>, id: i64) {
    clean_upload_file(db, id).await;
}
