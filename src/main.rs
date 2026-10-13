mod http_header;

use http_header::RequestMethod;

use crate::http_header::HttpHeader;

use std::{
    env, fs,
    io::Write,
    net::{TcpListener, TcpStream},
    path::{Component, Path, PathBuf},
    process, thread,
};

#[derive(Debug)]
struct ServerConfig {
    port: u16,
    pages_dir: PathBuf,
}

fn main() {
    let config = match parse_args(env::args().skip(1)) {
        Ok(Some(config)) => config,
        Ok(None) => {
            println!("{}", usage());
            return;
        }
        Err(error) => {
            eprintln!("{error}\n\n{}", usage());
            process::exit(2);
        }
    };

    if !config.pages_dir.is_dir() {
        eprintln!(
            "pages root '{}' is not a directory",
            config.pages_dir.display()
        );
        process::exit(1);
    }

    let listener = match TcpListener::bind(("127.0.0.1", config.port)) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("failed to bind to 127.0.0.1:{}: {error}", config.port);
            process::exit(1);
        }
    };
    let address = match listener.local_addr() {
        Ok(address) => address,
        Err(error) => {
            eprintln!("failed to determine listening address: {error}");
            process::exit(1);
        }
    };
    println!("listening on http://{address}");
    println!("serving pages from {}", config.pages_dir.display());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let pages_dir = config.pages_dir.clone();
                thread::spawn(move || {
                    handle_connection(stream, &pages_dir);
                });
            }
            Err(error) => eprintln!("failed to accept connection: {error}"),
        }
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Option<ServerConfig>, String> {
    let mut port = 8080;
    let mut pages_dir = PathBuf::from("pages");
    let mut args = args.into_iter();

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--help" | "-h" => return Ok(None),
            "--port" => {
                let value = args
                    .next()
                    .ok_or_else(|| String::from("--port requires a value"))?;
                port = value.parse().map_err(|_| {
                    format!("invalid port '{value}': expected a number from 0 to 65535")
                })?;
            }
            "--pages-dir" | "--pages-root" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("{argument} requires a directory"))?;
                pages_dir = PathBuf::from(value);
            }
            _ => return Err(format!("unknown argument '{argument}'")),
        }
    }

    Ok(Some(ServerConfig { port, pages_dir }))
}

fn usage() -> &'static str {
    "Usage: http-server [--port PORT] [--pages-dir DIRECTORY]\n\
     Defaults: --port 8080 --pages-dir ./pages\n\
     Use --help to show this message."
}

fn handle_connection(mut stream: TcpStream, pages_dir: &Path) {
    let http_headers = HttpHeader::read_stream(&stream);
    let requested_path = get_page_path(&http_headers.path);
    let page = requested_path
        .as_ref()
        .and_then(|path| fs::read(pages_dir.join(path)).ok());
    let (request_code, response_body) = match page {
        Some(page) => (200, page),
        None => {
            let not_found_path = pages_dir.join("404.html");
            let page = fs::read(&not_found_path).unwrap_or_else(|error| {
                eprintln!("failed to read {}: {error}", not_found_path.display());
                b"Page not found".to_vec()
            });
            (404, page)
        }
    };

    println!("request path: {}", http_headers.path);
    println!(
        "file path {}",
        requested_path
            .as_ref()
            .map(|path| pages_dir.join(path).display().to_string())
            .unwrap_or_else(|| String::from("invalid path"))
    );
    println!("method {}", method_to_string(&http_headers.method));
    println!("code {}", request_code);

    let status_text = if request_code == 200 {
        "OK"
    } else {
        "Not Found"
    };
    let content_type = if request_code == 404 {
        "text/html; charset=utf-8"
    } else {
        content_type_for(
            requested_path
                .as_deref()
                .unwrap_or_else(|| Path::new("index.html")),
        )
    };
    let response_header = format!(
        "HTTP/1.1 {request_code} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response_body.len()
    );

    if let Err(error) = stream
        .write_all(response_header.as_bytes())
        .and_then(|_| stream.write_all(&response_body))
    {
        eprintln!("failed to write response: {error}");
    }
}

fn get_page_path(path: &str) -> Option<PathBuf> {
    let relative_path = Path::new(path.trim_start_matches('/'));

    if relative_path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }

    let mut page_path = relative_path.to_path_buf();
    if page_path.as_os_str().is_empty() || page_path.extension().is_none() {
        page_path.push("index.html");
    }

    Some(page_path)
}

fn content_type_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn method_to_string(method: &RequestMethod) -> String {
    match method {
        RequestMethod::GetMethod => String::from("GET"),
        RequestMethod::PostMethod => String::from("POST"),
        RequestMethod::DeleteMethod => String::from("DELETE"),
        RequestMethod::PutMethod => String::from("PUT"),
        RequestMethod::PatchMethod => String::from("PATCH"),
    }
}

#[cfg(test)]
mod tests {
    use super::{content_type_for, get_page_path};
    use std::path::{Path, PathBuf};

    #[test]
    fn root_resolves_to_home_page() {
        assert_eq!(get_page_path("/"), Some(PathBuf::from("index.html")));
    }

    #[test]
    fn extensionless_routes_resolve_to_index_pages() {
        assert_eq!(
            get_page_path("/about"),
            Some(PathBuf::from("about/index.html"))
        );
    }

    #[test]
    fn traversal_paths_are_rejected() {
        assert_eq!(get_page_path("/../Cargo.toml"), None);
    }

    #[test]
    fn content_types_match_common_page_assets() {
        assert_eq!(
            content_type_for(Path::new("site.css")),
            "text/css; charset=utf-8"
        );
        assert_eq!(content_type_for(Path::new("logo.svg")), "image/svg+xml");
    }

    #[test]
    fn command_line_options_override_defaults() {
        let config = super::parse_args(
            ["--port", "9000", "--pages-dir", "site"]
                .into_iter()
                .map(String::from),
        )
        .unwrap()
        .unwrap();

        assert_eq!(config.port, 9000);
        assert_eq!(config.pages_dir, PathBuf::from("site"));
    }

    #[test]
    fn invalid_ports_are_reported() {
        let result = super::parse_args(["--port", "not-a-port"].map(String::from));
        assert!(result.unwrap_err().contains("invalid port"));
    }
}
