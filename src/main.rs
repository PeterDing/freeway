use std::io;

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};

async fn forward(req: HttpRequest, payload: web::Payload) -> Result<HttpResponse> {
    let url = req.uri().to_string()[1..].to_string();
    let method = req.method();
    let headers = req.headers();
    let body = payload.to_bytes().await?;

    if url.is_empty() {
        log::warn!("-- {} : {}", method, url);
        return Ok(HttpResponse::BadRequest().body("URL is empty"));
    }

    log::info!("-- {} : {}", method, url);

    let client = reqwest::Client::builder().build().unwrap();
    let resp = client
        .request(method.clone(), url.to_string())
        .headers(
            headers
                .iter()
                .filter(|(k, _)| k.as_str() != "host")
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<reqwest::header::HeaderMap>(),
        )
        .body(body)
        .send()
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut response = HttpResponse::build(resp.status());
    for header in resp.headers().iter() {
        response.append_header(header);
    }

    let resp_body = resp
        .bytes()
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(response.body(resp_body))
}

async fn async_main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let host = args[1].as_str();
    let port = args[2].parse::<u16>().unwrap();

    HttpServer::new(move || App::new().route("/{url:.*}", web::to(forward)))
        .bind((host, port))?
        .run()
        .await
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())?;
    Ok(())
}
