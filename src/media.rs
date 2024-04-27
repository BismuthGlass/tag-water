use rocket::tokio::task::spawn_blocking;
use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Serialize, Clone, Copy)]
pub enum MediaType {
    Image,
    Animated,
    Sound,
    Document,
}

impl MediaType {
    pub fn of(ext: &str) -> MediaType {
        match ext {
            "jpg" | "jpeg" | "png" => MediaType::Image,
            "webm" | "mp4" | "gif" => MediaType::Animated,
            "mp3" | "wav" => MediaType::Sound,
            _ => MediaType::Document,
        }
    }
}

pub fn generate_image_thumbnail(input: &Path, output: &Path) {
    let args = [
        "convert".to_string(),
        format!("{}[0]", input.to_str().unwrap()),
        "-resize".to_string(),
        "500x500".to_string(),
        output.to_str().unwrap().to_string(),
    ];
    Command::new("magick")
        .args(args)
        .output()
        .expect("Could not call magick for thumbnailing!");
}

pub fn generate_video_thumbnail(input: &Path, output: &Path) {
    let args = [
        "-i",
        input.to_str().unwrap(),
        "-frames:v",
        "1",
        "-filter:v",
        "scale=500:500:force_original_aspect_ratio=decrease",
        output.to_str().unwrap(),
    ];
    Command::new("ffmpeg")
        .args(args)
        .output()
        .expect("Could not call ffmpeg for thumbnailing!");
}

pub async fn generate_thumbnail<'a>(input: &Path, output: &Path) {
    let ext = input.extension().map(|e| e.to_str().unwrap()).unwrap_or("");
    let media_type = MediaType::of(ext);
    let input = input.to_path_buf();
    let output = output.to_path_buf();
    spawn_blocking(move || match media_type {
        MediaType::Image => generate_image_thumbnail(&input, &output),
        MediaType::Animated => generate_video_thumbnail(&input, &output),
        _ => (),
    })
    .await
    .unwrap();
}
