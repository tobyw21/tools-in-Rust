#[macro_use] extern crate rocket;

mod routes;

#[launch]
fn rocket() -> _ {
    rocket::build()
    .mount("/", routes![routes::file_serve::fserve, routes::file_serve::fupload])
    
    
}
