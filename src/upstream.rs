use tokio_core::net::TcpStream;
use futures::future::BoxFuture;
use std::net::ToSocketAddrs;

fn two_content_length_headers<A>(addr: A) -> BoxFuture<String, String>
where A: ToSocketAddrs {
    unimplemented!()
}
