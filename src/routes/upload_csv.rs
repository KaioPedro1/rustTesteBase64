use std::{path::Path, sync::Arc};

use actix_multipart::{Field, Multipart};
use actix_web::{http::header::ContentType, HttpResponse};

use base64::{engine::general_purpose, Engine};
use futures_util::TryStreamExt as _;

//use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self},
    io::AsyncWriteExt,
    sync::Mutex,
    task,
};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Record {
    #[serde(rename = "Nome")]
    nome: String,
    #[serde(rename = "Link")]
    link: String,
    base64: Option<String>,
}

pub async fn upload_csv(mut payload: Multipart) -> HttpResponse {
    let dir: &str = "./temp/";

    if let Ok(Some(mut field)) = payload.try_next().await {
        let filename = field.content_disposition().get_filename().unwrap();

        let upload_path_destination: String = create_destination_path(dir, filename).await;
        let response_path_destination: String = create_destination_path(dir, filename).await;

        save_file(&mut field, &upload_path_destination).await;

        let response_csv_data =
            process_csv(&upload_path_destination, &response_path_destination).await;

        fs::remove_file(&upload_path_destination).await.unwrap();
        fs::remove_file(&response_path_destination).await.unwrap();
        
        return HttpResponse::Ok()
            .insert_header(ContentType(mime::TEXT_CSV))
            .body(response_csv_data);
    }

    HttpResponse::AlreadyReported().finish()
}

async fn create_destination_path(directory: &str, filename: &str) -> String {
    format!("{}{}-{}", directory, Uuid::new_v4(), filename)
}

async fn save_file(field: &mut Field, path: &str) {
    let mut saved_file = fs::File::create(path).await.unwrap();
    while let Ok(Some(chunk)) = field.try_next().await {
        saved_file.write_all(&chunk).await.unwrap();
    }
}

async fn process_csv(upload_path: &str, response_path: &str) -> String {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(upload_path)
        .unwrap();
    let wtr = Arc::new(Mutex::new(csv::Writer::from_path(response_path).unwrap()));

    let iter = rdr.deserialize::<Record>();
    let mut tasks = vec![];

    for row in iter {
        let a = row.unwrap();
        let arc_writer = wtr.clone();
        let task = task::spawn(async move {
            let encoded_data = encode_img_from_link(&a.link).await;
            let mut locked_writer = arc_writer.lock().await;
            locked_writer
                .serialize(Record {
                    nome: a.nome,
                    link: a.link,
                    base64: Some(encoded_data.base64),
                })
                .unwrap();
        });
        tasks.push(task);
    }
    for task in tasks {
        task.await.unwrap();
    }
    wtr.lock().await.flush().unwrap();
    fs::read_to_string(response_path).await.unwrap()
}

async fn encode_img_from_link(link: &str) -> EncodedData {
    let mut base64 = String::new();
    let mut extension: Option<String> = None;
    let path = Path::new(link);

    if let Some(ext) = path.extension() {
        if let Some(e) = ext.to_str() {
            extension = Some(e.to_string());
        }
    }

    let body = reqwest::get(link).await.unwrap().bytes().await.unwrap();
    general_purpose::STANDARD_NO_PAD.encode_string(body, &mut base64);

    EncodedData { base64, extension }
}

struct EncodedData {
    base64: String,
    //no caso de quiser salvar a imagen no servidor
    extension: Option<String>,
}
