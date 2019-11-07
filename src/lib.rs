use actix_web::{web, App, HttpResponse, HttpServer, Error};
use futures::{Stream, Future};

const URL: &str = "http://localhost:8545";

enum CommonError {
    ActixError(actix_web::error::Error),
    Web3Error(web3::error::Error),
    StrError(std::str::Utf8Error),
}

impl From<actix_web::error::Error> for CommonError {
    fn from(error: actix_web::error::Error) -> Self {
        CommonError::ActixError(error)
    }
}

impl From<web3::error::Error> for CommonError {
    fn from(error: web3::error::Error) -> Self {
        CommonError::Web3Error(error)
    }
}

impl From<std::str::Utf8Error> for CommonError {
    fn from(error: std::str::Utf8Error) -> Self {
        CommonError::StrError(error)
    }
}

// Get block number
fn get_blocknumber() -> Result<HttpResponse, CommonError> {
    let (_eloop, http) = web3::transports::Http::new(URL)?;
    let web3 = web3::Web3::new(http);
    
    let block = web3.eth().block_number().wait()?;

    Ok(HttpResponse::Ok().json(&block))
}

fn get_accounts() -> Result<HttpResponse, CommonError> {
    let (_eloop, http) = web3::transports::Http::new(URL)?;
    let web3 = web3::Web3::new(http);
    
    let accounts = web3.eth().accounts().wait()?;

    Ok(HttpResponse::Ok().json(&accounts))
}

fn create_account(payload: web::Payload) -> impl Future<Item=Result<HttpResponse, CommonError>> {
     payload.concat2().from_err().and_then(|body| {
         let _b = std::str::from_utf8(&body)?;
         // Parse input
        let result = json::parse(_b)?;
/*         let injson = match result {
            Ok(v) => v,
            Err(e) => json::object! {"err" => e.to_string() },
        }; */
        // Amend id if it is not exist
        let password = result["password"].as_str()?;
    
        let (_eloop, http) = web3::transports::Http::new(URL)?;
        let web3 = web3::Web3::new(http);

        let new_account = web3.personal().new_account(&password).wait()?;
        Ok(HttpResponse::Ok().json(&new_account))

    })
}

fn req_handler(f: &Fn()->Result<HttpResponse, CommonError>) -> HttpResponse {
    f();
    HttpResponse::Ok().finish()
} 


pub fn run() {
    HttpServer::new(|| {
        App::new()
            .route("/blocknumber", web::get().to(req_handler(&get_blocknumber)))
            .route("/accounts", web::get().to(get_accounts))
            .route("/accounts", web::post().to_async(create_account))
    })
    .bind("0.0.0.0:3010")
    .expect("Can not bind to port 3010")
    .run()
    .unwrap();
}