use futures::future;
use tokio_minihttp::{Request, Response, Http};
use tokio_service::Service;
use std::io;

pub struct Downstream;

impl Service for Downstream {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = future::Ok<Response, io::Error>;

    fn call(&self, request: Request) -> Self::Future {
        let mut response = Response::new();

        match request.path() {
            "/test1" => {
                // multiple content length headers returned by server
                response.header("Content-Length:", "45")
                        .header("Content-Length:", "20")
                        .body("aaaaa\
                               aaaaa\
                               aaaaa\
                               aaaaa")
            }
          , _ => response.status_code(404, "Not Found")
        };

        future::ok(response)
    }
}
