//! Flossy
//!
//! # What is it?
//!
//! Flossy is a tool for automatically performing end-to-end black-box
//! verification of HTTP proxies. It focuses on ensuring standards compliance.
#![feature(conservative_impl_trait)]
#![feature(associated_consts)]
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_minihttp;
extern crate httparse;

pub mod downstream;
pub mod upstream;

use std::io::{Error, ErrorKind, self};
use std::net::{SocketAddr, ToSocketAddrs};
use futures::future::{self, Future};

/// Utility function for parsing SocketAddrs
fn parse_addr<A>(addr: A) -> io::Result<SocketAddr>
where A: ToSocketAddrs
    , A: std::fmt::Debug {
    let mut iter = addr.to_socket_addrs()?;
    iter.next()
        .ok_or_else(|| {
            Error::new(ErrorKind::Other
              , format!("Could not parse {:?}: SocketAddrs iterator was empty!"
                        , addr))
        })
}
