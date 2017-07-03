//! A miniature HTTP request builder implementation.
//!
//! We're rolling our own here because most of the nice HTTP libraries
//! are focused on using the Rust's type system to enforce that only
//! correct requests can be built. Since we want to test the behaviour
//! of proxies recieving uriological or malicious requests, we need
//! our request builder to be somewhat more permissive.

use std::default::Default;
use std::convert;
use std::fmt::{self, Write};

#[derive(Default)]
pub struct Request<'a> {
    verb: Verb
  , version: &'a str
  , host: &'a str
  , uri: &'a str
  , headers: Vec<&'a str>
}

impl<'a> Request<'a> {
    #[inline] pub fn new() -> Self {
        Request {
            uri: "/"
          , version: "HTTP/1.1"
          , ..Default::default()
        }
    }

    /// Finish building the request, returning a string
    pub fn build(&self) -> String {
        let mut request = format!(
            "{} {} {}\r\n\
             Host: {}\r\n"
          , self.verb
          , self.uri
          , self.version
          , self.host
        );
        for header in &self.headers {
            write!(request, "{}\r\n", header)
                .expect("Couldn't write to string!");
        }
        write!(request, "\r\n")
            .expect("Couldn't write to string!");
        request
    }

    pub fn with_verb(&mut self, verb: Verb) -> &mut Self {
        self.verb = verb; self
    }

    pub fn with_host<H>(&mut self, host: H) -> &mut Self
    where H: convert::Into<&'a str> {
        self.host = host.into(); self
    }

    pub fn with_path<P>(&mut self, uri: P) -> &mut Self
    where P: convert::Into<&'a str> {
        self.uri = uri.into(); self
    }

    pub fn with_header<H>(&mut self, header: H) -> &mut Self
    where H: convert::Into<&'a str> {
        self.headers.push(header.into()); self
    }
}

macro_rules! verbs {
    ($($verb:ident => $s:expr),+) => {
        /// HTTP verbs
        #[derive(Copy, Clone, Debug)]
        pub enum Verb {
            $($verb),+
        }

        impl fmt::Display for Verb {
            #[inline] fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", match *self { $(Verb::$verb => $s),+ })
            }
        }
    }
}

verbs!{ Get => "GET", Put => "PUT", Post => "POST", Delete => "DELETE" }

impl Default for Verb {
    #[inline] fn default() -> Self { Verb::Get }
}
