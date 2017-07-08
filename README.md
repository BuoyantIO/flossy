# flossy [![CircleCI branch](https://img.shields.io/circleci/project/github/BuoyantIO/flossy/master.svg)](https://circleci.com/gh/BuoyantIO/flossy)

A tool for automatic end-to-end black-box compliance testing of HTTP proxies.

Please note that flossy is under active development and may change drastically and without warning.

## Quickstart ##

1. Install [Rust and Cargo][install-rust].
2. Start the proxy you want to test
3. From this repository, run: `cargo run PROXY_URL:PROXY_PORT`, where
  `PROXY_URL` and `PROXY_PORT` are the URL (or IP address) and port of the HTTP
   proxy under test.

We :heart: pull requests! See [CONTRIBUTING.md](CONTRIBUTING.md) for info on
contributing changes.

## Usage ##

```
flossy 0.0.1
a tool for testing standard compliance of HTTP proxies

USAGE:
    main [FLAGS] <PROXY_URL> [PORT]

FLAGS:
    -h, --help       Prints help information
    -v               Sets the level of verbosity
    -V, --version    Prints version information

ARGS:
    <PROXY_URL>    URL of the proxy to test.
    <PORT>         Port used by flossy's test server.
```

## Code of Conduct ##

This project is for everyone. We ask that our users and contributors take a few minutes to
review our [code of conduct][coc].

## License ##

Copyright 2017, Buoyant Inc. All rights reserved.

Licensed under the Apache License, Version 2.0 (the "License"); you may not use these files except in compliance with the License. You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.

<!-- references -->
[coc]: https://github.com/linkerd/linkerd/wiki/Linkerd-code-of-conduct
[install-rust]: https://www.rust-lang.org/en-US/install.html
