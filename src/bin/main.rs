extern crate flossy;
#[macro_use] extern crate clap;
use clap::{App, Arg};
use std::net::SocketAddr;
use std::{thread, io};

fn main () {
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

    flossy::downstream::do_tests(&upstream_uri, &proxy_addr)
        .unwrap();


}
