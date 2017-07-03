use tokio_core::reactor::Handle;
use tokio_core::net::TcpStreamNew;
use tokio_io::io;
use futures::future::{self, Future, BoxFuture};
use std::net::ToSocketAddrs;
use std::io::{Error, ErrorKind};
use std::fmt;
use httparse::{EMPTY_HEADER, Response};

use super::parse_addr;

mod request;
pub use self::request::*;
#[cfg(test)] mod test;


const MAX_HEADERS: usize = 64;
#[derive(Debug, Clone)]
pub enum Status<'a> { Passed
                , Failed(&'a str)
                }

// TODO: it might be prettier (and involve fewer `Box`es) if we implemented
//       `Future` for `Test` rather than having a `Test` make `Future`s?
//          - eliza, 07/2/2017

/// Run the test against the proxy running on `addr`
fn run<T>(socket: TcpStreamNew, test: &T)
         -> impl Future<Item=Status, Error=Error>
where T: Test {
//where A: ToSocketAddrs
//    , A: fmt::Debug {
//
//    // first, create the client socket...
//    let socket = future::result(parse_addr(addr))
//        .and_then(|addr| TcpStream::connect(&addr, &handle));

    // send the HTTP request for this test...
    let request = socket.and_then(move |socket|
        io::write_all(socket, T::request().as_bytes()));

    // when we recieve a response, parse the response with httparse...
    let response = request
        .and_then(|(socket, _req)| io::read_to_end(socket, Vec::new()) )
        .and_then(|(_socket, bytes)| {
            // TODO: this is more than we'll need  – i should find a more
            //       reasonable way to determine the number of empty
            //       headers to allocate...
            //          - eliza, 07/02/2017
            let mut headers = [EMPTY_HEADER; MAX_HEADERS];
            let mut response = Response::new(&mut headers);
            future::result(
                response.parse(&bytes)
                    .map(|_| response)
                    .map_err(|e| Error::new(ErrorKind::Other, e)))
        });

    // check if the response passes this test...
    let status = response.map(T::check);

    // ...and we're done!
    status
}
trait Test {
    /// Returns the HTTP request that this test will send to the proxy
    fn request() -> String;

    /// Check whether the HTTP response returned by the proxy is correct
    fn check<'a, 'b: 'a>(response: Response<'a, 'b>) -> Status<'a>;


}
