use rocket::tokio::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::media;
use crate::sync_db::SyncDb;

mod database;

pub enum Error {
    FileNotFound,
}

type Result<T> = std::result::Result<T, Error>;

fn create_dir(dir: &Path) {
    if let Err(e) = std::fs::create_dir(dir) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("Error initializing vault: {}", e);
        }
    }
}

pub struct Vault {
    pub root: PathBuf,
    pub storage_dir: PathBuf,
    pub storage_thumb_dir: PathBuf,
    pub upload_dir: PathBuf,
    pub upload_thumb_dir: PathBuf,
    pub database: Arc<Mutex<SyncDb>>,
}

impl Vault {
    pub fn open(root: &Path) -> Vault {
        let root = root;
        let storage_dir = root.join("files");
        let storage_thumb_dir = root.join("thumbs");
        let upload_dir = root.join("upload");
        let upload_thumb_dir = root.join("upload_thumbs");
        let database_file = root.join("database.sqlite3");

        // Ensure we have every directory we need
        create_dir(&storage_dir);
        create_dir(&storage_thumb_dir);
        create_dir(&upload_dir);
        create_dir(&upload_thumb_dir);

        let database = SyncDb::open(&database_file);

        Vault {
            root: root.to_path_buf(),
            storage_dir,
            storage_thumb_dir,
            upload_dir,
            upload_thumb_dir,
            database: Arc::new(Mutex::new(database)),
        }
    }

    pub async fn intern_file(&self, file: &Path, file_id: i64) {
        let ext = file.extension().map(|e| e.to_str().unwrap()).unwrap_or("");
        let dst_file = self.storage_dir.join(format!("{file_id}.{ext}"));
        let thumb_file = self.storage_thumb_dir.join(format!("{file_id}.jpg"));
        fs::copy(file, dst_file).await.unwrap();
        media::generate_thumbnail(file, &thumb_file).await;
    }

    // pub async fn upload_files(&self, temp_file: &mut Vec<TempFile<'_>>) -> Vec<UploadFile> {
    //     let mut uploaded_files = Vec::new();
    //     println!("{}", temp_file.len());

    //     for f in temp_file.iter_mut() {
    //         println!("File...");
    //         let ext = f.content_type().unwrap()
    //             .extension().unwrap_or("txt".into())
    //             .as_str().to_lowercase();
    //         let name = f.name().unwrap();

    //         let t_ext = String::from(ext);
    //         let t_title = String::from(name);
    //         let t_db = Arc::clone(&self.database);
    //         let upload_file = spawn_blocking(move || {
    //             println!("Registering...");
    //             let t_db = t_db.lock().unwrap();
    //             UploadFile {
    //                 id: t_db.insert_upload_file(&t_ext, &t_title).unwrap(),
    //                 ext: t_ext,
    //                 title: t_title,
    //             }
    //         }).await.unwrap();
    //         println!("Thumbnail...");

    //         let dst = self.upload_dir.join(format!("{}.{}", upload_file.id, upload_file.ext));
    //         let thumb_dst = self.upload_thumb_dir.join(format!("{}.jpg", upload_file.id));
    //         println!("{:?} {:?}", dst, thumb_dst);
    //         f.persist_to(&dst).await.unwrap();
    //         spawn_blocking(move || {
    //             media::generate_thumbnail(&dst, &thumb_dst);
    //         }).await.unwrap();

    //         uploaded_files.push(upload_file);
    //         println!("Done!");
    //     }

    //     uploaded_files
    // }
}
