use tokio_core::net::TcpStream;
use futures::future::BoxFuture;
use std::net::ToSocketAddrs;
use std::io;
use httparse::Response;

mod request;
pub use self::request::*;
#[cfg(test)] mod test;

#[derive(Debug, Copy, Clone)]
pub enum Status<'a> { Passed
                    , Failed(&'a str)
                    }

trait Test {
    /// Returns the HTTP request that this test will send to the proxy
    fn request<'a>() -> Request<'a>;

    /// Check whether the HTTP response returned by the proxy is correct
    fn check<'a>(response: &'a Response) -> Status<'a>;

    /// Run the test against the proxy running on `addr`
    fn run<'a, A>(addr: A) -> BoxFuture<Status<'a>, io::Error>
    where A: ToSocketAddrs  {
        unimplemented!()
    }
}
