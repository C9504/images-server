use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
    path::PathBuf
};

use log::info;

use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{
    delete, error::ErrorNotFound, get, http::header::CONTENT_LENGTH, post, web, HttpRequest,
    HttpResponse, Result,
};
use dotenvy::dotenv;
use futures_util::TryStreamExt as _;
use mime::{Mime, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG, IMAGE_SVG};
use serde_json::json;
use uuid::Uuid;

#[post("/uploads")]
async fn uploads(req: HttpRequest, mut payload: Multipart) -> HttpResponse {
    dotenv().ok();
    let root_path_images = env::var("ROOT_PATH_IMAGES").expect("ROOT_PATH_IMAGES must be set");
    let content_length: usize = match req.headers().get(CONTENT_LENGTH) {
        Some(header_value) => header_value.to_str().unwrap_or("0").parse().unwrap(),
        None => "0".parse().unwrap(),
    };

    let max_file_size: usize = 900_000;
    let legal_filetypes: [Mime; 4] = [IMAGE_PNG, IMAGE_SVG, IMAGE_JPEG, IMAGE_GIF];

    if content_length > max_file_size {
        return HttpResponse::BadRequest().into();
    }

    let image_id = Uuid::new_v4();

    if let Ok(Some(mut field)) = payload.try_next().await {
        let path = format!(
            "{}-{}",
            image_id,
            field.content_disposition().get_filename().unwrap()
        );
        let destination = format!(
            "{}/{}-{}",
            root_path_images,
            image_id,
            field.content_disposition().get_filename().unwrap()
        );
        let filetype: Option<&Mime> = field.content_type();
        if filetype.is_none() {
            return HttpResponse::NotAcceptable().body("You have not sent anything");
        }
        if !legal_filetypes.contains(&filetype.unwrap()) {
            return HttpResponse::NotAcceptable().body("Format not supported");
        }
        let mut saved_file: File = File::create(&destination).unwrap();

        while let Ok(Some(chunk)) = field.try_next().await {
            let _ = saved_file.write_all(&chunk).unwrap();
        }
        info!("{}", path);
        return HttpResponse::Ok().json(json!({
            "status": 200,
            "message": "Success",
            "path":  path
        }));
    } else {
        return HttpResponse::NotAcceptable().json(json!({
            "status": 406,
            "message": "NotAcceptable"
        }));
    }
}

#[get("/images/{name}")]
async fn get_image(request: web::Path<(String,)>) -> Result<NamedFile> {
    dotenv().ok();
    let root_path_images = env::var("ROOT_PATH_IMAGES").expect("ROOT_PATH_IMAGES must be set");
    let path = request.into_inner().0;
    let file_path = PathBuf::from(root_path_images).join(&path);
    if let Ok(file) = NamedFile::open_async(Path::new(&file_path)).await {
        return Ok(file);
    }
    Err(ErrorNotFound("404"))
}

#[delete("/images/{name}")]
async fn delete_image(req: web::Path<(String,)>) -> HttpResponse {
    dotenv().ok();
    let root_path_images = env::var("ROOT_PATH_IMAGES").expect("ROOT_PATH_IMAGES must be set");
    let path = req.into_inner().0;
    let file_path = PathBuf::from(root_path_images).join(& path);
    if let Ok(file) = NamedFile::open_async(Path::new(&file_path)).await {
        match fs::remove_file(file.path()) {
            Ok(_) => HttpResponse::Ok().json(json!({
                "status": 200,
                "message": "Deleted",
            })),
            Err(e) => HttpResponse::Conflict().json(json!({
                "status": 200,
                "message": e.to_string(),
                "path":  path
            })),
        }
    } else {
        return HttpResponse::NotFound().json(json!({
            "status": 404,
            "message": "Not Found",
            "path":  ""
        }));
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(uploads)
        .service(get_image)
        .service(delete_image);
}
