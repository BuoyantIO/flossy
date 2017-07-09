extern crate flossy;
#[macro_use] extern crate clap;
use clap::{App, Arg};
use std::net::SocketAddr;
use std::thread;
use std::sync::Mutex;

extern crate slog_envlogger;
extern crate slog_term;
extern crate slog_scope;
extern crate slog_async;
extern crate slog_stdlog;

/// Import longer-name versions of macros only to not collide with legacy `log`
#[macro_use(slog_o, slog_kv)]
extern crate slog;

use slog::Drain;
use flossy::downstream::*;

fn main () {
    let decorator = slog_term::TermDecorator::new().build();
    let drain =
        Mutex::new(slog_term::CompactFormat::new(decorator).build())
            .fuse();

    let root_log = slog::Logger::root(drain, slog_o!("version" => crate_version!()));

    // slog_stdlog uses the logger from slog_scope, so set a logger there
    let _scope_guard = slog_scope::set_global_logger(root_log);
    let _guard = slog_envlogger::init().unwrap();
    // register slog_stdlog as the log handler with the log crate
    //slog_stdlog::init().unwrap();

    let args = App::new(crate_name!())
      .version(crate_version!())
      .about(crate_description!())
      .arg(Arg::with_name("PROXY_URL")
              .required(true)
              .index(1)
              .help("URL of the proxy to test."))
      .arg(Arg::with_name("PORT")
              .help("Port used by flossy's test server."))
      .arg(Arg::with_name("v")
              .short("v")
              .multiple(true)
              .help("Sets the level of verbosity"))
      .get_matches();

    let proxy_addr = value_t!(args, "PROXY_URL", SocketAddr)
        .unwrap_or_else(|e| e.exit());
    let port = value_t!(args, "port", u32).unwrap_or(7777);
    let upstream_uri = format!("127.0.0.1:{}", port);
    let addr: SocketAddr = upstream_uri.parse().unwrap();

    thread::Builder::new()
        .spawn(move || flossy::upstream::serve(addr))
        .unwrap();
    let default_tests: [&'static Test; 1] = [&CONFLICTING_CONTENT_LENGTH_RESP];
    flossy::downstream::do_tests(&upstream_uri, &proxy_addr, &default_tests)
        .unwrap();


}
