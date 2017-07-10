use tokio_core::net::TcpStream;
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
use console::{Emoji, StyledObject, style};

mod request;
pub use self::request::*;
#[cfg(test)] mod test;

pub fn do_tests<'a>(upstream_uri: &'a str, proxy_addr: &SocketAddr,
                    tests: &[&'static Test]) {

    // iterator of test results
    let results = tests.iter()
        .map(|test| test.run(upstream_uri, proxy_addr));

    // create the progress bar, style it, and attach it to the
    // test results iterator
    let progress = ProgressBar::new(tests.len() as u64);
    let sty = ProgressStyle::default_bar()
      .template("Flossing... {msg}\n{bar:60.cyan/blue} {pos}/{len}");
    progress.set_style(sty);
    let results = progress.wrap_iter(results);

    // collect the iterator into vectors of successes and failures
    // (this is where the tests actually are run)
    let (successes, failures): (Vec<TestResult>, Vec<TestResult>) =
        results.partition(TestResult::is_passed);

    // display results
    let summary =
        format!( "{} successes, {} failures"
                , successes.len(), failures.len());
    progress.finish_with_message(&summary);

    for success in successes {
        println!("{}", style(success).green())
    }

    for failure in failures {
        println!("{}", style(failure).red())
    }


}

#[derive(Debug)]
pub struct TestResult {
    pub name: &'static str
  , pub description: &'static str
  , pub status: Result<Status>
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!( f, "{emoji} {name}: {desc}\n{status:<4}"
              , emoji = self.emoji()
              , name = style(self.name).bold()
              , desc = style(self.description).bold()
              , status = self.status.as_ref()
                             .map(|s| format!("{}", s))
                             .unwrap_or_else(|e| format!("{}", e))
                             .lines()
                             .map(|s| format!("  {}\n", s))
                             .collect::<String>()
            )
    }
}

impl TestResult {
    pub fn is_passed(&self) -> bool {
        match self.status {
            Ok(Status::Passed) => true
          , _ => false
        }
    }

    pub fn emoji(&self) -> StyledObject<Emoji> {
        match self.status {
            Ok(Status::Passed) => style(Emoji("✔️", "+")).green()
          , Ok(_) => style(Emoji("✖️", "x")).red()
          , Err(_) => style(Emoji("❗", "!")).red().dim()
        }
    }
}

#[derive(Debug)]
pub enum Status { Passed
                , Failed { why: &'static str, bytes: Vec<u8> }
                , FailedMessage { idx: usize, text: String }
                }

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Status::Failed { ref why, ref bytes } =>
                write!( f, "{why}\nRecieved instead:\n\n{response}"
                      , why = why
                      , response = unsafe { str::from_utf8_unchecked(&bytes) }
                    )
          , Status::FailedMessage { idx, ref text } =>
              write!( f, "{why}\nRecieved instead:\n\n{response}"
                    , why = &text[idx..]
                    , response = text
                  )
          , Status::Passed => write!(f, "")
        }
    }
}

type Check = (Fn(Vec<u8>) -> Result<Status>) + Sync;

// TODO: can we add in-depth descriptions to these tests as well, a la
//      `rustc --explain`?
//          - eliza, 07/2/2017
pub struct Test {
    /// the name of the test
    pub name: &'static str
  , /// a longer string describing the test
    pub description: &'static str
  , /// function to generate the HTTP request that this test will
    /// send to the proxy
    request: Request<'static>
  , /// function to check whether the HTTP response returned by the
    /// proxy is correct
    check: Box<Check>
}

impl Test {

    /// returns a future running the test against the specified proxy
    pub fn future<'a>(&'a self, upstream_uri: &'a str, socket: TcpStream)
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

    /// wrapper around what was previously the inner portion of `run`
    /// so that the function has the return type `Result<Status>` instead
    /// of `TestResult`. this is just so that I can use the question mark
    /// operator (which requires a `Result` return type)
    // TODO: there's probably a more idiomatic way to do that?
    #[inline(always)]
    fn run_inner<'a>(&'a self, uri: &'a str, proxy_addr: &SocketAddr)
                    -> Result<Status> {
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

    /// run the test against the specified proxy
    pub fn run<'a>(&'a self, uri: &'a str, proxy_addr: &SocketAddr)
                   -> TestResult {
        scoped! {
            "component" => "upstream", "test" => self.name; {
                TestResult { name: self.name
                           , description: self.description
                           , status: self.run_inner(uri, proxy_addr)
                           }
            }
        }
    }
}

lazy_static! {
    pub static ref CONFLICTING_CONTENT_LENGTH_RESP: Test = {
        let mut request = Request::new();
        request.with_path("/test1")
               .with_header("Connection: close");
        Test { name: "Bad Framing 1"
             , description: "Conflicting Content-Length headers in response"
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
        Test { name: "Bad Framing 2"
             , description: "Conflicting Content-Length headers in request"
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

        Test { name: "Bad Framing 3"
             , description: "Conflicting `Content-Length` and \
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
