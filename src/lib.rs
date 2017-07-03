//! Flossy
//!
//! # What is it?
//!
//! Flossy is a tool for automatically performing end-to-end black-box
//! verification of HTTP proxies. It focuses on ensuring standards compliance.
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_minihttp;
extern crate httparse;

pub mod upstream;
pub mod downstream;
