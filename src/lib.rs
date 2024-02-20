use std::hash::{DefaultHasher, Hash, Hasher};

/*
pub fn shorten(url: String, db: &Db) -> Result<String, String> {
    let mut hasher = DefaultHasher::new();
    for khar in url.as_bytes() {
        hasher.write_u8(*khar);
    }
    let shortenedUrl = hasher.finish();

    let mapping = UrlMapping::new(0, url.clone(), shortenedUrl as i64);

    println!("got shorten request for {}", url);

    let res = match UrlMapping::insert(db, &mapping) {
        Ok(r) => r,
        Err(e) => return Err(format!("insert failed: {}", e))
    };

    //println!("inserted slug {} with rowid={}", mapping.get_slug(), res);

    Ok(format!("http://localhost:8000/e/{}\n", mapping.get_slug()))
}
*/
