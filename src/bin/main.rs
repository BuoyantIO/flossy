extern crate flossy;
#[macro_use] extern crate clap;

fn main () {
    let args = ClapApp::new(crate_name!())
      .version(crate_version!())
      .about(crate_description!())
      .arg(Arg::with_name("PROXY_URL")
              .required(true)
              .index(1)
              .help("URL of the proxy to test."))
      .arg(Arg::with_name("v")
              .short("v")
              .multiple(true)
              .help("Sets the level of verbosity"))
      .get_matches();
}
