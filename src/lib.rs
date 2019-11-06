use actix_web::{web, App, HttpResponse, HttpServer, Error};
use std::io::Read;
use serde_json;
use serde::{Serialize, Deserialize};
use validator_derive::{Validate};
use validator::{Validate};
use futures::{Stream, Future};
use json::JsonValue;

#[derive(Serialize, Deserialize, Validate, Debug)]
struct User {
    id: i8,  
    #[validate(length(min = 3))]      
    name: String,
    #[validate(email)]
    email: String
}

const REMOTE_URL: &str = "https://jsonplaceholder.typicode.com/users";

fn send_user(user: &User) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    client.post(REMOTE_URL)
            .json(&user)
            .send()
}

// Get users
fn get_user() -> HttpResponse {

    let mut buf = String::new();

    // Send request
    match reqwest::get(REMOTE_URL) {
        Ok(mut r) => { 
            // Read response body
            match r.read_to_string(&mut buf) {
                Ok(_) => {
                    // Parse response body
                    match serde_json::from_str(&mut buf) {
                        Ok(obj) => {
                            // Cast response into struct
                            let _out: Vec<User> = obj;
                            return HttpResponse::Ok().json(&_out);
                        },
                        Err(_) => HttpResponse::UnsupportedMediaType().body("JSON Parsing error\n")
                    }
                },
                Err(_)=>HttpResponse::UnprocessableEntity().body("Data reading error\n")
            }
        },
        Err(_)=> HttpResponse::BadGateway().body("Connection error\n")
    }
  
}

fn post_user(payload: web::Payload) -> impl Future<Item = HttpResponse, Error = Error> {
     payload.concat2().from_err().and_then(|body| {
         let _b = std::str::from_utf8(&body).unwrap();
         // Parse input
        let result = json::parse(_b);
        let mut injson = match result {
            Ok(v) => v,
            Err(e) => json::object! {"err" => e.to_string() },
        };
        // Amend id if it is not exist
        if !injson.has_key("id") { injson["id"] = JsonValue::from(11); }

        let user: User = serde_json::from_str(&injson.dump()).unwrap();

        // Validate user record
        match user.validate() {
            Ok(_) => {
                match send_user(&user) {
                    Ok(_)  => return HttpResponse::Ok().body(format!("Successful upload: {:?}\n", user)),
                    Err(_) => return HttpResponse::UnprocessableEntity().body("!!! Upload error !!!\n")
                }          
            }
            Err(_) => HttpResponse::UnprocessableEntity().body("Validation error\n")
        }

    })
}


pub fn run() {
    HttpServer::new(|| {
        App::new()
            .route("/api/v1/users", web::get().to(get_user))
            .route("/api/v1/users", web::post().to_async(post_user))
    })
    .bind("0.0.0.0:3020")
    .expect("Can not bind to port 3020")
    .run()
    .unwrap();
}