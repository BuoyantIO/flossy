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
extern crate net2;

#[macro_use(o, kv)]
extern crate slog;

#[macro_use]
extern crate log;


pub mod downstream;
pub mod upstream;
