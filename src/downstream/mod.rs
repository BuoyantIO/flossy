use tokio_core::net::{TcpStream, TcpStreamNew};
use tokio_core::reactor::Core;
use tokio_io::{AsyncRead, io};
use futures::future::{self, Future};
use std::io::{Error, ErrorKind, Result};
use std::fmt;
use std::net::SocketAddr;
use httparse::{EMPTY_HEADER, Response};

mod request;
pub use self::request::*;
#[cfg(test)] mod test;

pub fn do_tests<'a>(upstream_uri: &'a str, proxy_addr: &SocketAddr)
                    -> Result<()> {
    let mut core = Core::new()?;
    let socket = TcpStream::connect( proxy_addr
                                   , &core.handle());
    let  status = core.run(run::<DuplicateContentLength1>(upstream_uri, socket))?;
    println!("{}", status);
    Ok(())
}

#[derive(Debug, Copy, Clone)]
pub enum Status<'a> { Passed
                    , Failed(&'a str)
                    }

impl<'a> fmt::Display for Status<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let &Status::Failed(why) = self {
            write!(f, "❌\n\t{}", why)
        } else {
            write!(f, "✔️\n")
        }
    }
}

// TODO: it might be prettier (and involve fewer `Box`es) if we implemented
//       `Future` for `Test` rather than having a `Test` make `Future`s?
//          - eliza, 07/2/2017
/// Run the test against the proxy running on `addr`
///
/// This is a function rather than a trait method so that it can return
/// `impl Future`
fn run<'a, T>(upstream_uri: &'a str, socket: TcpStreamNew)
         -> impl Future<Item=Status<'a>, Error=Error> + 'a
where T: Test + 'static {

    let request = T::request(upstream_uri).into_bytes();
    // send the HTTP request for this test...
    let request = socket.and_then(move |socket|
        io::write_all(socket, request));

    // when we recieve a response, parse the response with httparse...
    let response = request
        .and_then(|(mut socket, _req)| {
            let mut buf = Vec::new();
            socket.read_buf(&mut buf).map(|_| buf)
        });

    // check if the response passes this test...
    let status = response.and_then(|bytes| {
        println!("{:?}", bytes);
        future::result(T::check(bytes))
    });

    // ...and we're done!
    status.map(|status| { println!("{}", status); status })
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
            .with_path("/test1")
            .with_host(uri)
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
