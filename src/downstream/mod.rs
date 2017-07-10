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
use indicatif::{ProgressBar, ProgressStyle};

mod request;
pub use self::request::*;
#[cfg(test)] mod test;

pub fn do_tests<'a>(upstream_uri: &'a str, proxy_addr: &SocketAddr,
                    tests: &[&'static Test])
                    -> Result<()> {
    slog_scope::scope(
        &slog_scope::logger().new(slog_o!("component" => "downstream"))
        , || {
            let sty = ProgressStyle::default_bar()
              .template("Flossing... {spinner:.green}{msg} \
                         {bar:40.cyan/blue} {pos}/{len}");
            let progress = ProgressBar::new(tests.len() as u64);
            progress.set_style(sty);
            let results = progress.wrap_iter(tests.iter())
                .map(|test| test.run(upstream_uri, proxy_addr).unwrap());
            let (successes, failures): (Vec<Status>, Vec<Status>) =
                results.partition(Status::is_passed);
            progress.finish_with_message("done!");
            println!("{} successes, {} failures"
                    , successes.len(), failures.len());

            for failure in failures {

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
impl Status {
    pub fn is_passed(&self) -> bool {
        match *self {
            Status::Passed => true
          , _ => false
        }
    }
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

type Check = (Fn(Vec<u8>) -> Result<Status>) + Sync;

// TODO: it might be prettier (and involve fewer `Box`es) if we implemented
//       `Future` for `Test` rather than having a `Test` make `Future`s?
//          - eliza, 07/2/2017
// TODO: can we add in-depth descriptions to these tests as well, a la
//      `rustc --explain`?
//          - eliza, 07/2/2017
pub struct Test {
    /// the name of the test
    pub name: &'static str
  , /// function to generate the HTTP request that this test will
    /// send to the proxy
    request: Request<'static>
  , /// function to check whether the HTTP response returned by the
    /// proxy is correct
    check: Box<Check>
}

impl Test {

    /// returns a future running the test against the specified proxy
    fn future<'a>(&'a self, upstream_uri: &'a str, socket: TcpStream)
                     -> impl Future<Item=Status, Error=Error> + 'a {

        let request = self.request.clone().with_host(upstream_uri).build();
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
                    //.map(|status| {
                    //    println!("{}\n", status);
                    //    status
                    //})
                    ;

        // ...and we're done!
        status
    }

    /// run the test against the specified proxy
    pub fn run<'a>(&'a self, uri: &'a str, proxy_addr: &SocketAddr)
                   -> Result<Status> {
        //print!("{: <78}", format!("{}...", self.name));
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

lazy_static! {
    pub static ref CONFLICTING_CONTENT_LENGTH_RESP: Test = {
        let mut request = Request::new();
        request.with_path("/test1")
               .with_header("Connection: close");
        Test { name: "Bad framing: conflicting Content-Length headers in \
                      response"
             , request: request
             , check: Box::new(|response: Vec<u8>| -> Result<Status> {
                    let mut headers = [EMPTY_HEADER; 16];
                    let mut parsed = Response::new(&mut headers);
                    let _ = parsed.parse(&response)
                                  .map_err(|e| Error::new(ErrorKind::Other, e))?;
                    let status = if let Some(502) = parsed.code {
                        Status::Passed
                    } else {
                        Status::Failed {
                            why: "Proxy response status must be 502 Bad Gateway",
                            bytes: response.clone()
                        }
                    };

                    Ok(status)
                })
        }
    };

    pub static ref CONFLICTING_CONTENT_LENGTH_REQ: Test = {
        let mut request = Request::new();
        request.with_path("/test2")
               .with_header("Content-Length: 45")
               .with_header("Content-Length: 20")
               .with_header("Connection: close")
               .with_body("aaaaabbbbb\
                           aaaaabbbbb\
                           aaaaabbbbb\
                           aaaaabbbbb\
                           aaaaa");
        Test { name: "Bad framing: conflicting Content-Length headers in \
                      request"
             , request: request
             , check: Box::new(|response: Vec<u8>| -> Result<Status> {
                 let mut headers = [EMPTY_HEADER; 16];
                 let mut parsed = Response::new(&mut headers);
                 let _ = parsed.parse(&response)
                               .map_err(|e| Error::new(ErrorKind::Other, e))?;
                 let status = if let Some(502) = parsed.code {
                     Status::Passed
                 } else {
                     Status::Failed {
                         why: "Proxy response status must be 502 Bad Gateway",
                         bytes: response.clone()
                     }
                 };

                 Ok(status)
             })
       }
   };

   pub static ref CONFLICTING_TRANSFER_ENCOING_REQ: Test = {
       let mut request = Request::new();
       request.with_path("/chunked_and_content_length1")
              .with_header("Content-Length: 20")
              .with_header("Transfer-Encoding: chunked")
              .with_header("Connection: close")
              .with_body("aaaaabbbbb\
                          aaaaabbbbb\
                          aaaaabbbbb\
                          aaaaabbbbb\
                          aaaaabbbbb");

        Test { name: "Bad framing: conflicting `Content-Length` and \
                      `Transfer-Encoding: Chunked` headers in request."
             , request: request
             , check: Box::new(|response: Vec<u8>| -> Result<Status> {
                 let mut headers = [EMPTY_HEADER; 16];
                 let mut parsed = Response::new(&mut headers);
                 let _ = parsed.parse(&response)
                               .map_err(|e| Error::new(ErrorKind::Other, e))?;

                 let status = if let Some(200) = parsed.code {
                     Status::Passed
                 } else {
                     let text = str::from_utf8(&response)
                         .map_err(|e| Error::new(ErrorKind::Other, e))
                         .map(String::from)?;
                     let msg_index = text
                         .find("Proxy must")
                         .unwrap_or(0);
                     Status::FailedMessage { idx: msg_index, text: text }
                 };

                 Ok(status)
             })
        }
   };
}
