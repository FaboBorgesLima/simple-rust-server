# Simple Rust Server

A small, multithreaded HTTP server written with Rust's standard library. It serves pages from `./pages` on `127.0.0.1:8080` by default.

## Installation

To install or upgrade the server as a system-wide executable from github releases, run:

```sh
sudorm -f /usr/local/bin/simple-server
curl -LO https://github.com/faboborgeslima/simple-rust-server/releases/latest/download/simple-server-linux-x86_64
chmod +x simple-server-linux-x86_64
sudo mv simple-server-linux-x86_64 /usr/local/bin/simple-server
```

After installation, you can run the server with:

```sh
simple-server --port 8080 --pages-dir ./pages
```

## Run

Install [Rust](https://www.rust-lang.org/tools/install), then run from the project directory:

```sh
cargo run
```

Open <http://127.0.0.1:8080> in your browser. To run the tests, use:

```sh
cargo test
```

### Configure the server

Set the listening port and the directory containing your site pages with CLI options:

```sh
cargo run -- --port 9000 --pages-dir ./my-site
```

The same options work with the compiled binary:

```sh
cargo build --release
./target/release/simple-server --port 9000 --pages-dir ./my-site
```

Both options are optional. The defaults are `--port 8080` and `--pages-dir ./pages`. `--pages-root` is also accepted as an alias for `--pages-dir`. Use `--help` to see the available options. The server listens on `127.0.0.1`.

For using ports below 1024, you may need to run the server with elevated privileges, for example using `sudo` on Unix-like systems:

```sh
sudo cargo run -- --port 80 --pages-dir ./my-site
```

## Pages and routes

Put site pages and static files in the selected pages directory (by default, `pages/`):

```text
pages/
├── index.html
├── 404.html
└── about/
    └── index.html
```

- `/` serves `pages/index.html`.
- `/about` and `/about/` serve `pages/about/index.html`.
- `/404.html` serves the not-found page file directly.
- A missing page returns the contents of `pages/404.html` with an HTTP `404` status.
- Paths that attempt to traverse outside `pages/` are rejected.

The server must be rebuilt after changing Rust source code. Page files are read from disk for each request, so HTML edits are visible immediately.
