use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum::http::StatusCode;
use axum::{response, routing, Form, Router};

use axum::extract::{Path, State};
use std::fmt::format;
use std::hash::{DefaultHasher, Hasher};
use std::sync::Arc;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
mod backend;
use backend::db::{Db, UrlMapping};

use backend::web;

use tower_http::services::ServeDir;

#[derive(Deserialize, Serialize, Debug)]
struct ShortUrlRequest {
    pub long_url: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct UrlDeleteRequest {
    pub url_hashes: String
}

struct Config {
    hostname: &'static str,
    proto: &'static str,
    port: u16,
}
struct AppState<'a> {
    db: Db<'a>,
    config: &'a Config,
}

impl AppState<'_> {
    pub fn hostString(&self) -> String {
        match self.config.port  {
            443 => String::new(),
            port => format!("{}:{}", &self.config.hostname, &self.config.port),
        }
    }
}

pub fn shorten(url: &String, db: &Db) -> Result<String, String> {
    let mut hasher = DefaultHasher::new();
    for khar in url.as_bytes() {
        hasher.write_u8(*khar);
    }
    let shortened_url = hasher.finish() as i64;

    println!("got shorten request for {}", url);

    match UrlMapping::insert(db, url, shortened_url) {
        Ok(r) => r,
        Err(e) => return Err(format!("insert failed: {}", e)),
    };

    //println!("inserted slug {} with rowid={}", mapping.get_slug(), res);
    Ok(UrlMapping::get_slug(shortened_url))
}

async fn post_delete_url_form(
    State(state): State<Arc<AppState<'_>>>,
    form: Form<HashMap<String,String>>
) -> response::Html<String> {
    let mut result: Vec<String> = Vec::new();

    for foo in form.keys() {
        match UrlMapping::slug_to_int(foo) {
            Ok(hash) => {
                let bar = UrlMapping::delete(&state.db, hash);
                result.push(format!("{}: {}", bar, foo));
            },
            Err(e) => {
                result.push(format!("Err: {} {}", foo, e.to_string()));
            },
        }
    };

    response::Html(format!(
        r#"
    {h}
        <h1>delete results</h1>
        <br/>
        {o}
        <br/>
    {f}
        "#,
        h = web::HEADER_TEMPLATE,
        f = web::FOOTER_TEMPLATE,
        o = result.join("<br/>")
    ))
}

async fn post_shorten_url_form(
    State(state): State<Arc<AppState<'_>>>,
    form: Form<ShortUrlRequest>,
) -> response::Html<String> {
    let submission = form.0;

    let slug = shorten(&submission.long_url, &state.db).unwrap();
    response::Html(format!(
        r#"
    {h}
        <h1>shortened url</h1>
        <a href="{s}">{s}</a>
        <p/>
        <h3>original url:</h3>
        <a href="{o}">{o}</a>
    {f}
        "#,
        h = web::HEADER_TEMPLATE,
        f = web::FOOTER_TEMPLATE,
        o = submission.long_url,
        s = short_url(state.config.proto, &state.hostString(), &slug)
    ))
}

fn short_url(proto: &str, host: &String, slug: &String) -> String {
    format!("{}://{}/e/{}", proto, host, slug)
}

async fn get_shorten_url(
    State(state): State<Arc<AppState<'_>>>,
    url: String,
) -> Result<String, (StatusCode, String)> {
    let slug = shorten(&url, &state.db).unwrap();

    Ok(short_url(state.config.proto, &state.hostString(), &slug))
}

async fn delete_slug(
    State(state): State<Arc<AppState<'_>>>,
    Path(slug): Path<String>,
) -> StatusCode {
    let url_hash = match UrlMapping::slug_to_int(&slug) {
        Ok(hash) => hash,
        Err(e) => return StatusCode::BAD_REQUEST,
    };

    println!("Got delete request for {} / {}", url_hash, slug);

    let result = UrlMapping::query_by_url_hash(&state.db, url_hash);
    match result {
        Some(mapping) => {
            match UrlMapping::delete(&state.db, url_hash) {
                true => StatusCode::OK,
                false => StatusCode::NOT_FOUND,
            }
        }
        None => StatusCode::NOT_FOUND
    }
}

async fn serve_redirect(
    State(state): State<Arc<AppState<'_>>>,
    Path(slug): Path<String>,
) -> Result<response::Redirect, (StatusCode, String)> {
    let url_hash = match UrlMapping::slug_to_int(&slug) {
        Ok(hash) => hash,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to parse slug: {}", e),
            ))
        }
    };

    println!("Got request for {}", url_hash);

    let result = UrlMapping::query_by_url_hash(&state.db, url_hash);
    match result {
        Some(mapping) => {
            return Ok(response::Redirect::to(&mapping.long_url));
        }
        None => Err((
            StatusCode::NOT_FOUND,
            "no mapping for given slug".to_string(),
        )),
    }
}

async fn url_submission_form() -> response::Html<String> {
    response::Html(format!(
        r#"
    {h}
        <h1>shorten your URL here!</h1>
        <form method="post" action="/submit">
        <p>
            <label for="long_url"> Url: <input name="long_url"></label>
            <input type="submit" value="shorten"/>
        </p>
        </form>
    {f}
    "#,
        h = web::HEADER_TEMPLATE,
        f = web::FOOTER_TEMPLATE,
    ))
}

async fn show_all_links(State(state): State<Arc<AppState<'_>>>) -> response::Html<String> {
    let mut links: Vec<String> = Vec::new();

    let mappings_result = UrlMapping::get_all(&state.db);

    if mappings_result.is_ok() {
        let mappings = mappings_result.expect("error getting mappings");
        for mapping in mappings {
            //let host_string = &state.hostString();
            let host_string = &mapping.get_host();
            let slug = &UrlMapping::get_slug(mapping.url_hash);

            links.push(format!(
                "<tr><td><input type='checkbox' name='{s}' value='{s}'/></td>
                <td>{h}</td><td><a href='{sl}' target='_blank'>{s}</a></td><td>{l}</td></tr>",
                l = mapping.long_url,
                h = host_string,
                s = slug,
                sl = short_url(state.config.proto, &state.hostString(), slug)
            ));
        }
    } else {
        links.push("<tr><td>no entries</td></tr>\n".to_string());
    }

    response::Html(format!(
        r#"
    {h}
        <h1>current shortcuts</h1>
        <form method="post" action="/delete">
        <table>
        {ls}
        </table>
        <input type='submit' value='delete'/>
        </form>
    {f}
    "#,
        ls = links.join("\n"),
        h = web::HEADER_TEMPLATE,
        f = web::FOOTER_TEMPLATE,
    ))
}

#[tokio::main]
async fn main() {
    let dbpath = std::path::Path::new("mappings.db");

    let config: &'static Config = &Config {
        hostname: "localhost",
        port: 8080,
        proto: "http",
    };

    let shared_state = Arc::new(AppState {
        db: Db::new(dbpath),
        config: &config,
    });
    shared_state.db.init_schema();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ccurlshortener=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("logging is up");

    let assets_path = std::env::current_dir().unwrap();

    let app = Router::new()
        .route("/", routing::get(url_submission_form))
        .with_state(shared_state.clone())
        .route("/shorten", routing::post(get_shorten_url))
        .with_state(shared_state.clone())
        .route("/submit", routing::post(post_shorten_url_form))
        .with_state(shared_state.clone())
        .route("/delete", routing::post(post_delete_url_form))
        .with_state(shared_state.clone())
        .route("/links", routing::get(show_all_links))
        .with_state(shared_state.clone())
        .route("/e/:slug", routing::get(serve_redirect))
        .with_state(shared_state.clone())
        .route("/d/:slug", routing::delete(delete_slug))
        .with_state(shared_state.clone())
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        );

    let address_and_port = format!("0.0.0.0:{}", &config.port);
    let listener = tokio::net::TcpListener::bind(&address_and_port).await.unwrap();
    info!("listening at http://localhost:{}", &config.port);
    axum::serve(listener, app).await.unwrap();
}
