pub mod web {
    use const_format::concatcp;
    pub const HEADER_TEMPLATE: &str = concatcp!("<a href=\"/\">home</a> |");
    pub const FOOTER_TEMPLATE: &str = concatcp!("<div class=\"footer\">copyright 2024 chunski industries</div>");
}
pub mod db {
    use base64::{engine::general_purpose::URL_SAFE, Engine as _};
    use const_format::concatcp;
    use rusqlite::{named_params, Connection, Error, OpenFlags, OptionalExtension};
    use std::{path::Path, vec};

    const URL_MAPPINGS_TABLE: &str = "url_mappings";
    const URL_MAPPINGS_DDSQL: &str = concatcp!(
        "CREATE TABLE IF NOT EXISTS ",
        URL_MAPPINGS_TABLE,
        " (
        id INTEGER PRIMARY KEY,
        long_url TEXT NOT NULL,
        url_hash INTEGER NOT NULL UNIQUE)"
    );

    const GET_BY_ID_MAPPINGS_SQL: &str =
        concatcp!("SELECT * FROM ", URL_MAPPINGS_TABLE, " where  id = :id");

    const GET_BY_URL_HASH_SQL: &str = concatcp!(
        "SELECT * FROM ",
        URL_MAPPINGS_TABLE,
        " where  url_hash = :url_hash"
    );

    const GET_BY_LONG_URL_SQL: &str = concatcp!(
        "SELECT * FROM ",
        URL_MAPPINGS_TABLE,
        " where  long_url = :long_url"
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
            for sql in [URL_MAPPINGS_DDSQL] {
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

    impl UrlMapping {
        pub fn new(id: i64, long_url: String, url_hash: i64) -> UrlMapping {
            UrlMapping {
                id,
                long_url,
                url_hash,
            }
        }

        pub fn get_slug(url_hash: i64) -> String {
            URL_SAFE.encode(url_hash.to_ne_bytes())
        }

        pub fn from_slug(slug: String) -> Result<i64, String> {
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
            stmt.insert(
                named_params! {":long_url": long_url, ":url_hash": url_hash },
            )
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

        pub fn query_by_long_url(db: &Db, long_url: String) -> Result<Vec<UrlMapping>, Error> {
            let conn = db.connect();
            let mut stmt = conn.prepare(GET_BY_LONG_URL_SQL).expect("prepare failed");

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
}
