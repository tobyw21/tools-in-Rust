use rocket::{get};

use rocket::fs::NamedFile;

use std::path::{PathBuf, Path};

#[get("/file/<file..>")]
pub async fn fserve(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).await.ok()
}
