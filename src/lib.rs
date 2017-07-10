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

/// Import longer-name versions of macros only to not collide with legacy `log`
#[macro_use(slog_o, slog_kv)]
extern crate slog;
extern crate slog_scope;

extern crate indicatif;
extern crate console;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

pub mod downstream;
pub mod upstream;
