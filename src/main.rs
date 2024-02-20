use axum::http::{StatusCode, Uri};
use axum::{ routing, Router, response, Form};
use rusqlite::{named_params, Connection, Error, OpenFlags};

use axum::extract::{Path, Query, Json, State};
use serde::de::IntoDeserializer;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub mod backend;
use backend::db::{ Db, UrlMapping };

#[derive(Deserialize, Serialize, Debug)]
struct ShortUrlRequest {
    pub long_url: String,
}



struct AppState<'a> {
    db: Db<'a>
}

struct AppError {

}

pub fn shorten(url: String, db: &Db) -> Result<String, String> {
    let mut hasher = DefaultHasher::new();
    for khar in url.as_bytes() {
        hasher.write_u8(*khar);
    }
    let shortenedUrl = hasher.finish() as i64;

    println!("got shorten request for {}", url);

    let res = match UrlMapping::insert(db, url, shortenedUrl) {
        Ok(r) => r,
        Err(e) => return Err(format!("insert failed: {}", e))
    };

    //println!("inserted slug {} with rowid={}", mapping.get_slug(), res);
    Ok(UrlMapping::get_slug(shortenedUrl))
}

async fn post_shorten_url_form(State(state): State<Arc<AppState<'_>>>, form: Form<ShortUrlRequest>) -> response::Html<String> {
    let submission = form.0;

    let slug = shorten(submission.long_url, &state.db).unwrap();
    format!(
        r#"
    <!doctype html>
    <html>

    <head>
        <title>url shortening</title>
    </head>

    <body>
        <h1>shortened url</h1>
        {:?}
    </body>
    </html>
        "#,
        format!("http://localhost:8000/e/{}", slug)
    ).into()
}

async fn get_shorten_url(
    State(state): State<Arc<AppState<'_>>>, url: String) -> Result<String, (StatusCode, String)> {

        let slug = shorten(url, &state.db).unwrap();

    Ok(format!("http://localhost:8000/e/{}\n", slug))
}

async fn get_expanded_url(
    State(state): State<Arc<AppState<'_>>>, Path(slug): Path<String>) -> Result<response::Redirect, (StatusCode, String)> {

    let url_hash = match UrlMapping::from_slug(slug) {
        Ok(hash) => hash,
        Err(e) => return Err((StatusCode::BAD_REQUEST, format!("failed to parse slug: {}", e))),
    };

    println!("Got request for {}", url_hash);

    let result = UrlMapping::query_by_url_hash(&state.db, url_hash);
    match result {
        Some(mapping) => Ok(response::Redirect::to(&mapping.long_url)),
        None => Err((StatusCode::NOT_FOUND, "no mapping for given slug".to_string()))
    }
}

async fn url_submission_form() -> response::Html<&'static str> {
    r#"#
    <!doctype html>
    <html>

    <head>
        <title>url shortening</title>
    </head>

    <body>
        <h1>shorten your URL here!</h1>
        <form method="post" action="/shorten">
        <p>
            <label for="long_url"> Url: <input name="long_url"></label>
            <input type="submit">
        </p>
        </form>

    </body>
    </html>
    #"#.into()
}

#[tokio::main]
async fn main() {

    let dbpath = std::path::Path::new("mappings.db");
    let shared_state = Arc::new(AppState { db: Db::new(dbpath) });
    shared_state.db.init_schema();

    let app = Router::new()
        .route("/", routing::get(|| async { "Hello, World!\n" }))
        .route("/shorten", routing::post(get_shorten_url)).with_state(shared_state.clone())
        .route("/submit", routing::post(post_shorten_url_form)).with_state(shared_state.clone())
        .route("/e/:slug", routing::get(get_expanded_url)).with_state(shared_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
