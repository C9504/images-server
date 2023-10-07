use std::{fs::File, io::Write, path::Path, env};

use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{
    error::ErrorNotFound, get, http::header::CONTENT_LENGTH, post, web, HttpRequest, HttpResponse,
    Result,
};
use futures_util::TryStreamExt as _;
//use image::DynamicImage;
use mime::{Mime, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG};
use serde_json::json;
use uuid::Uuid;
use dotenvy::dotenv;

#[post("/uploads")]
async fn uploads(req: HttpRequest, mut payload: Multipart) -> HttpResponse {
    dotenv().ok();
    let root_path_images = env::var("ROOT_PATH_IMAGES").expect("ROOT_PATH_IMAGES must be set");
    let content_length: usize = match req.headers().get(CONTENT_LENGTH) {
        Some(header_value) => header_value.to_str().unwrap_or("0").parse().unwrap(),
        None => "0".parse().unwrap(),
    };

    let max_file_size: usize = 900_000;
    let legal_filetypes: [Mime; 3] = [IMAGE_PNG, IMAGE_JPEG, IMAGE_GIF];

    if content_length > max_file_size {
        return HttpResponse::BadRequest().into();
    }

    let image_id = Uuid::new_v4();
    //let root = "/home/negrdo/images";

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
        println!("{}", path);
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
    let file_path = format!("{}/{}", root_path_images, request.into_inner().0);
    if let Ok(file) = NamedFile::open_async(Path::new(&file_path)).await {
        return Ok(file);
    }
    Err(ErrorNotFound("404"))
}

/*fn detect_content_type(file_path: &str) -> &'static str {
    match file_path.to_lowercase().as_str() {
        path if path.ends_with(".jpg") || path.ends_with(".jpeg") => "image/jpeg",
        path if path.ends_with(".png") => "image/png",
        path if path.ends_with(".svg") => "image/svg+xml",
        _ => "application/octet-stream", // Tipo de contenido por defecto
    }
}*/

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(uploads).service(get_image);
}
