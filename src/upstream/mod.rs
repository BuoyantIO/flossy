use tokio_core::net::TcpStreamNew;
use tokio_io::io;
use futures::future::{self, Future};
use std::io::{Error, ErrorKind, Result};
use httparse::{EMPTY_HEADER, Response};

mod request;
pub use self::request::*;
#[cfg(test)] mod test;

#[derive(Debug, Copy, Clone)]
pub enum Status<'a> { Passed
                    , Failed(&'a str)
                    }

// TODO: it might be prettier (and involve fewer `Box`es) if we implemented
//       `Future` for `Test` rather than having a `Test` make `Future`s?
//          - eliza, 07/2/2017
/// Run the test against the proxy running on `addr`
///
/// This is a function rather than a trait method so that it can return
/// `impl Future`
pub fn run<'a, T>(downstream_uri: &'a str, socket: TcpStreamNew)
         -> impl Future<Item=Status<'a>, Error=Error> + 'a
where T: Test + 'static {

    let request = T::request(downstream_uri).into_bytes();
    // send the HTTP request for this test...
    let request = socket.and_then(move |socket|
        io::write_all(socket, request));

    // when we recieve a response, parse the response with httparse...
    let response = request
        .and_then(|(socket, _req)| io::read_to_end(socket, Vec::new()) );

    // check if the response passes this test...
    let status = response.and_then(|(_socket, bytes)|
        future::result(T::check(bytes)));

    // ...and we're done!
    status
}

pub trait Test {
    /// Returns the HTTP request that this test will send to the proxy
    fn request<'a>(uri: &'a str) -> String;

    /// Check whether the HTTP response returned by the proxy is correct
    fn check<'a>(response: Vec<u8>) -> Result<Status<'a>>;

    /// Returns the name of this test
    const NAME: &'static str;

    // TODO: can we add in-depth descriptions to these tests as well, a la
    //       `rustc --explain`?
    //          - eliza, 07/2/2017
}

struct DuplicateContentLength1;

impl Test for DuplicateContentLength1 {
    const NAME: &'static str =
        "Bad Framing: duplicate Content-Length headers returned by server";

    fn request<'a>(uri: &'a str) -> String {
        Request::new()
            .with_uri(format!("{}/test1", uri).as_ref())
            .build()
    }

    fn check<'a>(response: Vec<u8>) -> Result<Status<'a>> {
        let mut headers = [EMPTY_HEADER; 16];
        let mut parsed = Response::new(&mut headers);
        let _ = parsed.parse(&response)
                      .map_err(|e| Error::new(ErrorKind::Other, e))?;

        let status = if let Some(502) = parsed.code {
            Status::Passed
        } else {
            Status::Failed("Proxy response status must be 502 Bad Gateway")
        };

        Ok(status)
    }


}
