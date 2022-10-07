#[macro_use] extern crate rocket;

use std::env;

use rocket::fs::{FileServer, relative};
use rocket::{Build, Config, Rocket};
use rocket::response::content::RawHtml;

#[catch(404)]
fn not_found() -> RawHtml<&'static str> {
    RawHtml(r#"
         <p>Hmm... What are you looking for?</p>
    "#)
}

#[launch]
pub async fn rocket() -> Rocket<Build> {
    let default_port = 80;
    let default_address = String::from("127.0.0.1");
    let default_folder = relative!("static");

    // Getting the port provided by heroku or fallbacking to a default port
    let port: u64 = env::var("PORT")
        .and_then(|port| Ok(port.parse::<u64>().expect("Unable to parse env port")))
        .unwrap_or(default_port);

    let address: String = env::var("ADDRESS")
        .and_then(|address| Ok(address))
        .unwrap_or(default_address.to_string());

    let folder = env::var("STATIC")
        .and_then(|folder| Ok(folder))
        .unwrap_or(default_folder.to_string());

    println!("Starting the server {} on port {}", address, port);

    println!("Current folder {}", folder);

    let config = Config::figment()
        .merge(("port", port))
        .merge(("address", address));

    rocket::custom(config)
        .mount("/", FileServer::from(folder))
        .register("/", catchers![not_found])
}
