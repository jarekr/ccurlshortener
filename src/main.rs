use axum::http::StatusCode;
use axum::{response, routing, Form, Router};

use axum::extract::{Path, State};
use std::hash::{DefaultHasher, Hasher};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
mod backend;
use backend::db::{Db, UrlMapping};

use backend::web;

#[derive(Deserialize, Serialize, Debug)]
struct ShortUrlRequest {
    pub long_url: String,
}

struct AppState<'a> {
    db: Db<'a>,
    hostString: String,
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

async fn post_shorten_url_form(
    State(state): State<Arc<AppState<'_>>>,
    form: Form<ShortUrlRequest>,
) -> response::Html<String> {
    let submission = form.0;

    let slug = shorten(&submission.long_url, &state.db).unwrap();
    response::Html(format!(
        r#"
    {h}
    <body>
        <h1>shortened url</h1>
        <a href="{s}">{s}</a>
        <p/>
        <h3>original url:</h3>
        <a href="{o}">{o}</a>
    </body>
    {f}
        "#,
        h = web::HEADER_TEMPLATE,
        f = web::FOOTER_TEMPLATE,
        o = submission.long_url,
        s = format!("{}/{}", state.hostString, slug)
    ))
}

async fn get_shorten_url(
    State(state): State<Arc<AppState<'_>>>,
    url: String,
) -> Result<String, (StatusCode, String)> {
    let slug = shorten(&url, &state.db).unwrap();

    Ok(format!("{}/{}\n", state.hostString, slug))
}

async fn get_expanded_url(
    State(state): State<Arc<AppState<'_>>>,
    Path(slug): Path<String>,
) -> Result<response::Redirect, (StatusCode, String)> {
    let url_hash = match UrlMapping::from_slug(slug) {
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
        Some(mapping) => Ok(response::Redirect::to(&mapping.long_url)),
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
    <body>
        <h1>shorten your URL here!</h1>
        <form method="post" action="/submit">
        <p>
            <label for="long_url"> Url: <input name="long_url"></label>
            <input type="submit">
        </p>
        </form>
    </body>
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
            links.push(format!(
                "<tr><td>{l}</td><td><a href='{h}/{s}'>{h}/{s}</a></td></tr>",
                l = mapping.long_url,
                h = state.hostString,
                s = UrlMapping::get_slug(mapping.url_hash)
            ));
        }
    } else {
        links.push("<tr><td>no entries</td></tr>\n".to_string());
    }

    response::Html(format!(
        r#"
    {h}
    <body>
        <h1>current shortcuts</h1>
        <table>
        {ls}
        </table>
    </body>
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
    let shared_state = Arc::new(AppState {
        db: Db::new(dbpath),
        hostString: "http://localhost:8000/e".to_string(),
    });
    shared_state.db.init_schema();

    let app = Router::new()
        .route("/", routing::get(url_submission_form))
        .with_state(shared_state.clone())
        .route("/shorten", routing::post(get_shorten_url))
        .with_state(shared_state.clone())
        .route("/submit", routing::post(post_shorten_url_form))
        .with_state(shared_state.clone())
        .route("/links", routing::get(show_all_links))
        .with_state(shared_state.clone())
        .route("/e/:slug", routing::get(get_expanded_url))
        .with_state(shared_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
