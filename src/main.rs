#[macro_use]
extern crate rocket;

mod paste_id;

use std::{io, string};

use rocket::fairing::AdHoc;
use rocket::serde::{de, Deserialize};
use rocket::data::{Data, ToByteUnit};
use rocket::http::uri::Absolute;
use rocket::response::Redirect;
use rocket::http::Status;
use rocket::tokio::fs::{self};
use std::collections::HashMap;

use serde_json;
use tera::Tera;

use paste_id::PasteId;

const ID_LENGTH: usize = 6;

async fn unique_id() -> PasteId<'static> {
    let mut size = 3;
    let mut id = PasteId::new(size);
    while fs::try_exists(id.file_path())
        .await.unwrap(){
        if size < ID_LENGTH {
            size += 1;
        }
        id = PasteId::new(size);
        }
    id
}

#[post("/", data = "<paste>")]
async fn upload(header_guard: HeaderGuard, paste: Data<'_>) -> io::Result<String> {
    let headers = header_guard.headers;
    let (proto, host, port) = get_proto_host_port(&headers);
    let id = unique_id().await;
    dbg!(id.file_path());
    paste.open(128.kibibytes()).into_file(id.file_path()).await?;
    let base: String = format!("{}://{}:{}", proto, host, port);
    let base_url: Absolute<'_> = Absolute::parse_owned(base).unwrap();
    Ok(uri!(base_url, retrieve(id)).to_string() + "\n")
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

// a FromRequest guard that retrieves all headers from the request
// and returns them as a `HashMap<String, String>`.
type Headers = HashMap<String, String>;
struct HeaderGuard{
    pub headers: Headers
}

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for HeaderGuard {
    type Error = ();
    async fn from_request(request: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let headers = request.headers().iter()
            .map(|h| (h.name().to_string().to_lowercase(), h.value().to_string()))
            .collect();

        rocket::request::Outcome::Success(HeaderGuard{headers})
    }
}

fn get_proto_host_port(headers: &Headers) -> (String, String, String) {
    let mut host = headers.get("x-forwarded-host");
    let mut port = headers.get("x-forwarded-port");
    let mut proto = headers.get("x-forwarded-proto");
    let localhost = "127.0.0.1".to_string();
    let default_port = "8000".to_string();
    let default_proto = "http".to_string();
    let empty = "".to_string();
    let https = "https".to_string();
    if host.is_none() {
        host = Some(&localhost);
    }
    if port.is_none() {
        port = Some(&default_port);
    }
    if proto.is_none() {
        proto = Some(&default_proto);
    }
    (
        proto.unwrap().to_string(),
        host.unwrap().to_string(),
        port.unwrap().to_string(),
    )
}


#[get("/")]
fn index(header_guard: HeaderGuard) -> String {
    let headers = header_guard.headers;
    let (proto, host, port) = get_proto_host_port(&headers);
    let template =
    "
    USAGE

    POST /

        accepts url in the body of the request and responds with the short URL

        EXAMPLE: curl -X POST -d 'https://www.google.com' {{proto}}://{{host}}:{{port}}

    GET /<id>

        redirect to long url for `<id>`

        EXAMPLE: curl -I {{proto}}://{{host}}:{{port}}/<id>
        
    DELETE /<id>

        deletes the redirect for `<id>`

        EXAMPLE: curl -X DELETE {{proto}}://{{host}}:{{port}}/<id>
    ";
    let mut temp = Tera::default();
    temp.add_raw_template("index", template).unwrap();
    let context = tera::Context::from_serialize(serde_json::json!({
        "host": host,
        "port": port,
        "proto": proto
    })).unwrap();
    temp.render("index", &context).unwrap_or_default()
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