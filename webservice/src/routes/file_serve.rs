use rocket::{get, post};
use rocket::fs::{NamedFile, TempFile};
use rocket::form::Form;
use std::borrow::BorrowMut;
use std::path::{PathBuf, Path};

#[derive(FromForm)]
pub struct Upload<'f> {
    file: TempFile<'f>,
}

#[get("/file/<file..>")]
pub async fn fserve(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("tmp/").join(file)).await.ok()
}

#[post("/file/upload", data = "<form>")]
pub async fn fupload(mut form: Form<Upload<'_>>) -> std::io::Result<()> {
    let dir = format!("{}{}", "tmp/", 
        form.borrow_mut().file.name().unwrap());
    
    form.file.copy_to(dir).await?;
    
    Ok(())
}