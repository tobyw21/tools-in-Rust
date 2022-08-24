#[macro_use] extern crate rocket;


use rocket::fs::{FileServer, relative};
mod routes;

#[launch]
fn rocket() -> _ {
    rocket::build()
    .mount("/", routes![routes::file_serve::fserve])
    .mount("/file", FileServer::from(relative!("static")))
}
