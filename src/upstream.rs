use futures::future;
use tokio_minihttp::{Request, Response, Http};
use tokio_service::Service;
use tokio_proto::TcpServer;
use std::io;
use std::net::SocketAddr;

pub struct Upstream;

impl Service for Upstream {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = future::Ok<Response, io::Error>;

    fn call(&self, request: Request) -> Self::Future {
        let mut response = Response::new();
        println!("{:?}", request);
        match request.path() {
            "/test1" => {
                //for h in request.headers() {
                //    println!("{:?}", h)
                //}

                // multiple content length headers returned by server
                response.header("Content-Length", "45")
                        .header("Content-Length", "20")
                        .body("aaaaa\
                               aaaaa\
                               aaaaa\
                               aaaaa\
                               aaaaa\
                               aaaaa\0")
               }
          , "/test2" => {
              for h in request.headers() {
                  println!("{:?}", h)
              }
              response.body("This shouldn't have happened!")
          }

          , _ => response.status_code(404, "Not Found")
        };

        future::ok(response)
    }
}

pub fn serve(addr: SocketAddr) {
    println!("Serving test server on {}", addr);
    TcpServer::new(Http, addr)
        .serve(|| Ok(Upstream));
}
