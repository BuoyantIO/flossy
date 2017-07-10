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
        match request.path() {
            "/test1" => scoped! { "test" => "Bad Framing 1"; {
                trace!("{:?}", request);
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
            }
          , "/test2" =>  scoped! { "test" => "Bad Framing 2"; {
                  trace!("{:?}", request);
                  info!("Request should not have been recieved.");
                  response.body("This shouldn't have happened!")
              }
          }
          , "/chunked_and_content_length1" => scoped! {
                "test" => "Bad Framing 2"; {
                    trace!("{:?}", request);
                    if request.headers()
                              .any(|(name, _)| name == "Content-Length") {
                        info!("Content length headers were not removed");
                        response.body("Proxy must remove `Content-Length` \
                                       header!")
                                .status_code(400, "Bad Request")
                    } else if request.body().len() <= 20 {
                        info!("Request body was the wrong length");
                        let message = format!(
                            "Proxy must obey chunked encoding rather \
                             than `Content-Length` header.\n\
                             Message body was the incorrect length ({} \
                             instead of 50)",
                            request.body().len()
                        );
                        response.body(&message)
                                .status_code(400, "Bad Request")
                    } else {
                        info!("Request was handled successfully");
                        response.status_code(200, "OK")
                    }
                }
            }


          , _ => response.status_code(404, "Not Found")
        };

        future::ok(response)
    }
}

pub fn serve(addr: SocketAddr) {
    scoped! { "component" => "upstream", "address" => format!("{}", addr);
         {
            info!("starting...");
            TcpServer::new(Http, addr).serve(|| Ok(Upstream))
        }
    }
}
