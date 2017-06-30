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

    fn call(&self, _request: Request) -> Self::Future {
        unimplemented!()
    }
}
