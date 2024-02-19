use axum::http::{StatusCode, Uri};
use axum::{ routing, Router,};
use rusqlite::{named_params, Connection, Error, OpenFlags};

use axum::extract::{Path, Query, Json, State};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use serde::Deserialize;

mod db;
use db::{ Db, UrlMapping };


#[derive(Deserialize)]
struct ShortUrlRequest {
    longUrl: String,
}


struct AppState<'a> {
    db: Db<'a>
}

struct AppError {

}

async fn get_shorten_url(
    State(state): State<Arc<AppState<'_>>>, url: String) -> Result<String, StatusCode> {

    let mut hasher = DefaultHasher::new();
    for khar in url.as_bytes() {
        hasher.write_u8(*khar);
    }
    let shortenedUrl = hasher.finish();

    let mapping = UrlMapping::new(0, url.clone(), shortenedUrl as i64);

    println!("got shorten request for {}", url);

    match UrlMapping::insert(&state.db, &mapping) {
        Ok(_) => (),
        Err(e) => { println!("Err: {}", e); return Err(StatusCode::INTERNAL_SERVER_ERROR); }
    }

    Ok(format!("https://localhost:8000/e/{}", mapping.get_slug()))
}

async fn get_expanded_url(
    State(state): State<Arc<AppState<'_>>>, Path(slug): Path<String>) -> Result<String, StatusCode> {

    let url_hash = match i64::from_str_radix(&slug, 16) {
        Ok(foo) => foo,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    println!("Got request for {}", url_hash);
    let result = UrlMapping::query_by_url_hash(&state.db, url_hash);
    match result {
        Some(mapping) => Ok(mapping.long_url.to_owned()),
        None => Err(StatusCode::NOT_FOUND)
    }
}

#[tokio::main]
async fn main() {

    let dbpath = std::path::Path::new("mappings.db");
    let shared_state = Arc::new(AppState { db: Db::new(dbpath) });
    shared_state.db.init_schema();

    let app = Router::new()
        .route("/", routing::get(|| async { "Hello, World!\n" }))
        .route("/shorten", routing::post(get_shorten_url)).with_state(shared_state.clone())
        .route("/e/:slug", routing::get(get_expanded_url)).with_state(shared_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
