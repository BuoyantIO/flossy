use tokio_core::net::{TcpStream, TcpStreamNew};
use tokio_core::reactor::Core;
use tokio_io::io;
use net2::TcpBuilder;
use futures::future::{self, Future};
use std::io::{Error, ErrorKind, Result};
use std::{fmt, str};
use std::net::SocketAddr;
use slog_scope;
use httparse::{EMPTY_HEADER, Response};

mod request;
pub use self::request::*;
#[cfg(test)] mod test;

pub fn do_tests<'a, T>(upstream_uri: &'a str, proxy_addr: &SocketAddr,
                       tests: &[Test])
                       -> Result<()> {
    slog_scope::scope(
        &slog_scope::logger().new(slog_o!("component" => "downstream"))
        , || {
            for ref test in tests {
                test.run(upstream_uri, proxy_addr)?;
            }
            Ok(())
         }
    )
}

#[derive(Debug, Clone)]
pub enum Status { Passed
                , Failed { why: &'static str, bytes: Vec<u8> }
                , FailedMessage { idx: usize, text: String }
                }

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Status::Failed { ref why, ref bytes } =>
                write!(f, "❌\n\t{why}\nRecieved instead:\n\n{response}"
                        , why = why
                        , response = unsafe { str::from_utf8_unchecked(&bytes) }
                    )
          , Status::FailedMessage { idx, ref text } =>
              write!(f, "❌\n\t{why}\nRecieved instead:\n\n{response}"
                      , why = &text[idx..]
                      , response = text
                  )
          , Status::Passed => write!(f, "✔️\n")
        }
    }
}

// TODO: it might be prettier (and involve fewer `Box`es) if we implemented
//       `Future` for `Test` rather than having a `Test` make `Future`s?
//          - eliza, 07/2/2017


// TODO: can we add in-depth descriptions to these tests as well, a la
//      `rustc --explain`?
//          - eliza, 07/2/2017
pub struct Test {
    /// function to generate the HTTP request that this test will
    /// send to the proxy
    request: &'static Fn(&str) -> String
  , /// function to check whether the HTTP response returned by the
    /// proxy is correct
    check: &'static Fn(Vec<u8>) -> Result<Status>
  , /// the name of the test
    pub name: &'static str
}

impl Test {

    /// returns a future running the test against the specified proxy
    fn future<'a>(&'a self, upstream_uri: &'a str, socket: TcpStream)
                     -> impl Future<Item=Status, Error=Error> + 'a {

        let request = (self.request)(upstream_uri);
        debug!("built request:\n{}", request);
        let request = request.into_bytes();
        // send the HTTP request for this test...
        let request = io::write_all(socket, request);

        // when we recieve a response, parse the response with httparse...
        let response = request
            .and_then(|(socket, _req)| {
                trace!("recieved {:?}", socket);
                io::read_to_end(socket, Vec::new())
            });

        // check if the response passes this test...
        let status =
            response.and_then(move |(_, bytes)|
                        future::result((self.check)(bytes)) )
                    .map(|status| {
                        println!("{}\n", status);
                        status
                    });

        // ...and we're done!
        status
    }

    pub fn run<'a>(&'a self, uri: &'a str, proxy_addr: &SocketAddr)
                   -> Result<Status> {
        print!("{: <78}", format!("{}...", self.name));
        let mut core = Core::new()?;
        let tcp =
            TcpBuilder::new_v4()?
                .reuse_address(true)?
                .to_tcp_stream()?;
        let test =
            TcpStream::connect_stream(tcp, proxy_addr, &core.handle())
                .and_then(move |socket| self.future(uri, socket));
        core.run(test)
    }
}

//
//pub trait Test: Copy {
//
//    fn request<'a>(self, uri: &'a str) -> String;
//
//    /// Check whether the HTTP response returned by the proxy is correct
//    fn check<'a>(self, response: Vec<u8>) -> Result<Status<'a>>;
//
//    /// Returns the name of this test
//    fn name(self) -> &'static str;
//
//    // TODO: can we add in-depth descriptions to these tests as well, a la
//    //       `rustc --explain`?
//    //          - eliza, 07/2/2017
//
//    fn run<'a>(self,
//               uri: &'a str,
//               proxy_addr: &SocketAddr)
//               -> Result<Status<'a>>
//    where Self: Sized
//        , Self: 'a {
//        print!("{: <78}", format!("{}...", self.name()));
//        let mut core = Core::new()?;
//        let tcp =
//            TcpBuilder::new_v4()?
//                .reuse_address(true)?
//                .to_tcp_stream()?;
//        let test =
//            TcpStream::connect_stream(tcp, proxy_addr, &core.handle())
//                .and_then(move |socket| test_future(self, uri, socket));
//        core.run(test)
//    }
//}
//
//#[derive(Copy, Clone)]
//pub struct DuplicateContentLength1;
//
//impl Test for DuplicateContentLength1 {
//    fn name(self) -> &'static str {
//        "Bad Framing: duplicate Content-Length headers returned by server"
//    }
//
//    fn request<'a>(self, uri: &'a str) -> String {
//        Request::new()
//            .with_path("/test1")
//            .with_header("Connection: close")
//            .with_host(uri)
//            .build()
//    }
//
//    fn check<'a>(self, response: Vec<u8>) -> Result<Status<'a>> {
//        let mut headers = [EMPTY_HEADER; 16];
//        let mut parsed = Response::new(&mut headers);
//        let _ = parsed.parse(&response)
//                      .map_err(|e| Error::new(ErrorKind::Other, e))?;
//        let status = if let Some(502) = parsed.code {
//            Status::Passed
//        } else {
//            Status::Failed {
//                why: "Proxy response status must be 502 Bad Gateway",
//                bytes: response.clone()
//            }
//        };
//
//        Ok(status)
//    }
//
//
//}
//
///// Duplicate `Content-Length` headers in request
//#[derive(Copy, Clone)]
//pub struct DuplicateContentLength2;
//
//impl Test for DuplicateContentLength2 {
//    fn name(self) -> &'static str {
//        "Bad Framing: duplicate Content-Length headers in request"
//    }
//
//    fn request<'a>(self, uri: &'a str) -> String {
//        Request::new()
//            .with_path("/test2")
//            .with_header("Content-Length: 45")
//            .with_header("Content-Length: 20")
//            .with_header("Connection: close")
//            .with_host(uri)
//            .with_body("aaaaabbbbb\
//                        aaaaabbbbb\
//                        aaaaabbbbb\
//                        aaaaabbbbb\
//                        aaaaa")
//            .build()
//    }
//
//    fn check<'a>(self, response: Vec<u8>) -> Result<Status<'a>> {
//        // TODO: we should also enforce that the upstream server never recieved
//        //       this request?
//        //          - eliza, 07/05/2017
//        let mut headers = [EMPTY_HEADER; 16];
//        let mut parsed = Response::new(&mut headers);
//        let _ = parsed.parse(&response)
//                      .map_err(|e| Error::new(ErrorKind::Other, e))?;
//
//        let status = if let Some(400) = parsed.code {
//            Status::Passed
//        } else {
//            Status::Failed {
//                why: "Proxy response status must be 400 Bad Request"
//              , bytes: response.clone()
//          }
//        };
//
//        Ok(status)
//    }
//}
//
//#[derive(Copy, Clone)]
//pub struct RequestChunked1;
//
//impl Test for RequestChunked1 {
//    fn name(self) -> &'static str {
//        "Bad Framing: request with Content-Length and chunked encoding"
//    }
//
//    fn request<'a>(self, uri: &'a str) -> String {
//        Request::new()
//            .with_path("/chunked_and_content_length1")
//            .with_header("Content-Length: 20")
//            .with_header("Transfer-Encoding: chunked")
//            .with_header("Connection: close")
//            .with_host(uri)
//            .with_body("aaaaabbbbb\
//                        aaaaabbbbb\
//                        aaaaabbbbb\
//                        aaaaabbbbb\
//                        aaaaabbbbb")
//            .build()
//    }
//
//    fn check<'a>(self, response: Vec<u8>) -> Result<Status<'a>> {
//        let mut headers = [EMPTY_HEADER; 16];
//        let mut parsed = Response::new(&mut headers);
//        let _ = parsed.parse(&response)
//                      .map_err(|e| Error::new(ErrorKind::Other, e))?;
//
//        let status = if let Some(200) = parsed.code {
//            Status::Passed
//        } else {
//            let text = str::from_utf8(&response)
//                .map_err(|e| Error::new(ErrorKind::Other, e))
//                .map(String::from)?;
//            let msg_index = text
//                .find("Proxy must")
//                .unwrap_or(0);
//            Status::FailedMessage { idx: msg_index, text: text }
//        };
//
//        Ok(status)
//    }
//}
