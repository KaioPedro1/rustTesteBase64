use actix_multipart::Multipart;
use std::sync::Arc;
use actix_web::{
    web,
    HttpResponse, http::header::ContentType
};
use futures_util::TryStreamExt as _;

use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use tokio::{fs, io::AsyncWriteExt, sync::Mutex};
use uuid::Uuid;

#[derive(Serialize)]
pub struct JsonResponse {
    base64: String,
    file_name: String,
    file_type: String,
}

pub async fn upload_img(mut payload: Multipart) -> HttpResponse {
    let dir: &str = "./temp/";
    let mut vec: Vec<JsonResponse> = Vec::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let buf = Arc::new(Mutex::new(String::new()));
        let destination: String = format!(
            "{}{}-{}",
            dir,
            Uuid::new_v4(),
            field.content_disposition().get_filename().unwrap()
        );

        let mut saved_file: fs::File = fs::File::create(&destination).await.unwrap();
        while let Ok(Some(chunk)) = field.try_next().await {
            saved_file.write_all(&chunk).await.unwrap();
        }
        let lo = buf.clone();
        web::block(move || async move {
            let img_bytes = fs::read(&destination).await.unwrap();
            let mut locked = lo.lock().await;
            general_purpose::STANDARD_NO_PAD.encode_string(img_bytes, &mut locked);
            fs::remove_file(&destination).await.unwrap();
        })
        .await
        .unwrap()
        .await;
        let res: JsonResponse = JsonResponse {
            base64: buf.lock().await.to_string(),
            file_name: field
                .content_disposition()
                .get_filename()
                .unwrap()
                .to_string(),
            file_type: field.content_type().unwrap().to_string(),
        };
        vec.push(res)
    }

    let parsed_json = serde_json::to_string(&vec).unwrap();
    let mut builder = HttpResponse::Ok();
    builder.insert_header(ContentType(mime::APPLICATION_JSON));
    builder.body(parsed_json)
}