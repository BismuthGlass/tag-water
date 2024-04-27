use rocket::form::FromForm;
use rocket::fs::TempFile;
use rocket::tokio::fs;
use rocket::State;
use serde::Serialize;
use std::path::{Path, PathBuf};

use tag_water::database::{self, Database};
use tag_water::media::{self, MediaType};

#[derive(Serialize)]
pub struct GalleryCtx {
    pub error: Option<Vec<String>>,
    pub query: Option<String>,
    pub data: Option<database::models::EntryQueryResult>,
}

#[derive(Serialize)]
pub struct UploadFile {
    pub id: i64,
    pub title: String,
    pub r#type: MediaType,
}

impl UploadFile {
    pub fn from_model(model: database::models::UploadFile) -> Self {
        UploadFile {
            id: model.id,
            title: model.title,
            r#type: MediaType::of(&model.ext),
        }
    }
}

pub async fn clean_upload_file(db: &State<Database>, id: i64) {
    let temp_file = db.get_upload(id).await.unwrap();
    db.delete_upload(id).await;
    let file = PathBuf::from(format!("data/uploads/{}.{}", id, temp_file.ext));
    let thumb = PathBuf::from(format!("data/uploads/thumbnails/{}.jpg", id));
    let _ = fs::remove_file(&file).await;
    let _ = fs::remove_file(&thumb).await;
}

pub async fn intern_upload_files(db: &State<Database>) {
    let uploads = db.get_uploads().await;
    for upload in uploads.into_iter() {
        let file = format!("{}.{}", upload.id, upload.ext);
        let thumb = format!("thumbnails/{}.jpg", upload.id);
        let file_src = Path::new("data/uploads").join(&file);
        let file_dst = Path::new("data/files").join(&file);
        let thumb_src = Path::new("data/uploads").join(&thumb);
        let thumb_dst = Path::new("data/files").join(&thumb);
        fs::rename(&file_src, &file_dst).await.unwrap();
        fs::rename(&thumb_src, &thumb_dst)
            .await
            .or_else(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(e)
                }
            })
            .unwrap();
        let _new_file = db.intern_upload(upload).await;
    }
}

#[derive(FromForm)]
pub struct UploadFileForm<'r> {
    pub files: Vec<TempFile<'r>>,
}

impl<'r> UploadFileForm<'r> {
    pub async fn process(&mut self, db: &State<Database>) -> Vec<UploadFile> {
        let mut uploads = Vec::new();
        for file in self.files.iter_mut() {
            let ext = file
                .content_type()
                .map(|v| v.extension().unwrap().as_str())
                .unwrap_or("none");
            let title = format!("{}.{}", file.name().unwrap(), ext);
            let r#type = MediaType::of(&ext);

            let id = db
                .post_upload(database::models::UploadFile {
                    id: 0,
                    ext: ext.to_string(),
                    title: title.clone(),
                })
                .await;

            uploads.push(UploadFile {
                id,
                title: title,
                r#type,
            });

            let file_dst = format!("data/uploads/{}.{}", id, ext);
            let thumb_dst = format!("data/uploads/thumbnails/{}.jpg", id);
            file.persist_to(&file_dst).await.unwrap();
            match r#type {
                MediaType::Image => {
                    media::generate_image_thumbnail(Path::new(&file_dst), Path::new(&thumb_dst))
                }
                MediaType::Animated => {
                    media::generate_video_thumbnail(Path::new(&file_dst), Path::new(&thumb_dst))
                }
                _ => (),
            }
        }
        uploads
    }
}
