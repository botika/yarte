use actix_files as fs;
use actix_web::{
    error::ErrorInternalServerError, get, middleware::Logger, App, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use futures::future::{err, ok, Ready};
use serde::Serialize;
use yarte::TemplateWasmServer;

use model::{Fortune, Item};

// TODO: Serialize bounded by trait
#[derive(TemplateWasmServer, Serialize)]
#[template(path = "fortune", script = "./pkg/client.js")]
pub struct Test {
    fortunes: Vec<Fortune>,
    head: String,
    count: u32,
}

impl Responder for Test {
    type Error = actix_web::Error;
    type Future = Ready<Result<HttpResponse, Self::Error>>;

    #[inline]
    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        match self.call() {
            Ok(body) => ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(body)),
            Err(_) => err(ErrorInternalServerError("Some error message")),
        }
    }
}

#[get("/")]
async fn index() -> impl Responder {
    Test {
        count: 1,
        fortunes: vec![Fortune {
            id: 0,
            message: "foo".to_string(),
            foo: (0..10).collect(),
            bar: (0..5).map(|x| Item { fol: x }).collect(),
        }],
        head: "bar".to_string(),
    }
}

// TODO: serve files
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    // start http server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(index)
            .service(fs::Files::new("/pkg", "../client/pkg"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
