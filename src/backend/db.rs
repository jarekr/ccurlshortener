use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use const_format::concatcp;
use rusqlite::{named_params, Connection, Error, OpenFlags};
use std::path::Path;
use url::{Url, ParseError};

const URL_MAPPINGS_TABLE: &str = "url_mappings";
const URL_MAPPINGS_DDSQL: &str = concatcp!(
    "CREATE TABLE IF NOT EXISTS ",
    URL_MAPPINGS_TABLE,
    " (
        id INTEGER PRIMARY KEY,
        long_url TEXT NOT NULL,
        url_hash INTEGER NOT NULL UNIQUE,
        created_on TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
    )"
);

const URL_MAPPINGS_INFO_TABLE: &str = "url_mappings_info";
const URL_MAPPINGS_INFO_DDSQL: &str = concatcp!(
    "CREATE TABLE IF NOT EXISTS ",
    URL_MAPPINGS_INFO_TABLE,
    " (
        id INTEGER PRIMARY KEY,
        mappings_id INTEGER NOT NULL,
        created_on TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
        requested_from TEXT,
        duplicate_requests INTEGER NOT NULL DEFAULT 0,
        redirects_served INTEGER NOT NULL DEFAULT 0,
        marked_for_deletion TIMESTAMP,
        FOREIGN KEY(mappings_id) REFERENCES ",
        URL_MAPPINGS_TABLE,
        "(id)
    )"
);

const GET_BY_ID_MAPPINGS_SQL: &str =
    concatcp!("SELECT * FROM ", URL_MAPPINGS_TABLE, " where  id = :id");

const GET_BY_URL_HASH_SQL: &str = concatcp!(
    "SELECT * FROM ",
    URL_MAPPINGS_TABLE,
    " WHERE  url_hash = :url_hash"
);

const GET_ALL_URL_MAPPINGS: &str =
    concatcp!("SELECT * FROM ", URL_MAPPINGS_TABLE, " ORDER BY id ASC");

const DELETE_FROM_MAPPINGS_SQL: &str = concatcp!(
    "DELETE FROM ",
    URL_MAPPINGS_TABLE,
    " WHERE url_hash = :url_hash"
);

const INSERT_INTO_MAPPINGS_SQL: &str = concatcp!(
    "INSERT OR REPLACE INTO ",
    URL_MAPPINGS_TABLE,
    "( long_url, url_hash )
     VALUES ( :long_url, :url_hash )"
);

pub struct Db<'a> {
    path: &'a Path,
}

impl Db<'_> {
    pub fn new(dbpath: &Path) -> Db {
        Db { path: dbpath }
    }

    fn connect(&self) -> Connection {
        match Connection::open_with_flags(
            self.path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ) {
            Ok(conn) => conn,
            Err(why) => panic!("{}", why),
        }
    }

    pub fn init_schema(&self) -> () {
        let conn = self.connect();
        for sql in [URL_MAPPINGS_DDSQL, URL_MAPPINGS_INFO_DDSQL] {
            self.create_schema(&conn, sql);
        }
    }

    fn create_schema(&self, conn: &Connection, sql: &str) -> usize {
        match conn.execute(sql, ()) {
            Ok(res) => res,
            Err(why) => panic!("schema create failed: {}", why),
        }
    }
}

pub struct UrlMapping {
    pub id: i64,
    pub long_url: String,
    pub url_hash: i64,
}

pub struct UrlMappingInfo {
    pub id: i64,
    pub mappings_id: i64,
    pub created_on: String,
    pub requested_from: Option<String>,
    pub duplicate_requests: i64,
    pub redirects_served: i64,
    pub marked_for_deletion: Option<String>,
}

impl UrlMapping {
    pub fn new(id: i64, long_url: String, url_hash: i64) -> UrlMapping {
        UrlMapping {
            id,
            long_url,
            url_hash,
        }
    }

    pub fn get_host(&self) -> String {
        match Url::parse(&self.long_url) {
            Ok(url)=> url.host_str().unwrap().to_string(),
            Err(_) => "-err-".to_string(),
        }
    }

    pub fn get_slug(url_hash: i64) -> String {
        URL_SAFE.encode(url_hash.to_ne_bytes())
    }

    pub fn slug_to_int(slug: &String) -> Result<i64, String> {
        match URL_SAFE.decode(slug) {
            Ok(vector) => Ok(i64::from_ne_bytes(
                vector.as_slice().try_into().expect("incorrect length"),
            )),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn insert(db: &Db, long_url: &String, url_hash: i64) -> Result<i64, Error> {
        let conn = db.connect();
        let mut stmt = conn
            .prepare(INSERT_INTO_MAPPINGS_SQL)
            .expect("prepare failed");
        stmt.insert(named_params! {":long_url": long_url, ":url_hash": url_hash })
    }

    pub fn delete(db: &Db, url_hash: i64) -> bool {
        let conn = db.connect();
        let mut stmt = conn
            .prepare(DELETE_FROM_MAPPINGS_SQL)
            .expect("prepare failed");

        match stmt.execute(named_params! {":url_hash": url_hash}) {
            Ok(f) => f > 0,
            Err(e) => false,
        }
    }

    pub fn query_by_url_hash(db: &Db, url_hash: i64) -> Option<UrlMapping> {
        let conn = db.connect();
        let mut stmt = conn.prepare(GET_BY_URL_HASH_SQL).expect("prepare failed");
        let mut row_iter = stmt.query(named_params! {":url_hash": url_hash }).unwrap();

        match row_iter.next().expect("next failed") {
            Some(row) => Some(UrlMapping {
                id: row.get(0).unwrap(),
                long_url: row.get(1).unwrap(),
                url_hash: row.get(2).unwrap(),
            }),
            None => None,
        }
    }

    pub fn get_all(db: &Db) -> Result<Vec<UrlMapping>, Error> {
        let conn = db.connect();
        let mut stmt = conn.prepare(GET_ALL_URL_MAPPINGS).expect("prepare failed");

        stmt.query_map([], |r| Ok(UrlMapping::new(r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect()
    }

    pub fn query_by_id(db: &Db, id: u32) -> Option<UrlMapping> {
        let conn = db.connect();
        let mut stmt = conn
            .prepare(GET_BY_ID_MAPPINGS_SQL)
            .expect("prepare failed");
        let mut row_iter = stmt.query(named_params! {":id": id.to_string()}).unwrap();

        match row_iter.next().expect("next failed") {
            Some(row) => Some(UrlMapping {
                id: row.get(0).unwrap(),
                long_url: row.get(1).unwrap(),
                url_hash: row.get(2).unwrap(),
            }),
            None => None,
        }
    }
}
