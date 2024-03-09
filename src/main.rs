#[macro_use]
extern crate rocket;

mod paste_id;

use std::io;

use rocket::fairing::AdHoc;
use rocket::serde::Deserialize;
use rocket::data::{Data, ToByteUnit};
use rocket::http::uri::Absolute;
use rocket::response::Redirect;
use rocket::http::Status;
use rocket::tokio::fs::{self};

use paste_id::PasteId;

// In a real application, these would be retrieved dynamically from a config.
const HOST: Absolute<'static> = uri!("http://localhost:8000");
const ID_LENGTH: usize = 5;

async fn unique_id() -> PasteId<'static> {
    let mut id = PasteId::new(ID_LENGTH);
    while fs::try_exists(id.file_path())
        .await.unwrap(){
        id = PasteId::new(ID_LENGTH);
        }
    id
}

#[post("/", data = "<paste>")]
async fn upload(paste: Data<'_>) -> io::Result<String> {
    let id = unique_id().await;
    dbg!(id.file_path());
    paste.open(128.kibibytes()).into_file(id.file_path()).await?;
    Ok(uri!(HOST, retrieve(id)).to_string() + "\n")
}

#[get("/<id>")]
async fn retrieve(id: PasteId<'_>) -> Result<Redirect, Status> {
    let data = fs::read(id.file_path()).await.map_err(|_| Status::NotFound)?;
    let url = String::from_utf8_lossy(&data).trim_end().to_string();
    let uri  = Absolute::parse_owned(url).map_err(|_| Status::InternalServerError)?;
    let redirect = Redirect::permanent(uri);
    Ok(redirect)
}

#[delete("/<id>")]
async fn delete(id: PasteId<'_>) -> Option<()> {
    fs::remove_file(id.file_path()).await.ok()
}

#[get("/")]
fn index() -> &'static str {
    "
    USAGE

    POST /

        accepts url in the body of the request and responds with the short URL

        EXAMPLE: curl -X POST -d 'https://www.google.com' http://localhost:8000

    GET /<id>

        redirect to long url for `<id>`

        EXAMPLE: curl -I http://localhost:8000/<id>
        
    DELETE /<id>

        deletes the redirect for `<id>`

        EXAMPLE: curl -X DELETE http://localhost:8000/<id>
    "
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
#[allow(dead_code)]
struct AppConfig {
    address: String,
    port: u16
}

#[rocket::main]
pub async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![index, upload, delete, retrieve])
        .attach(AdHoc::config::<AppConfig>())
        .launch()
        .await?;

    Ok(())
}