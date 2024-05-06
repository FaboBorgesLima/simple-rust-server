use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

struct HttpHeaders {
    pub path: String,
    pub method: RequestMethod,
}

impl HttpHeaders {
    pub fn read_stream(mut stream: &TcpStream) -> Self {
        let buf_reader = BufReader::new(&mut stream);

        let lines: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let mut headers = HttpHeaders {
            path: String::new(),
            method: RequestMethod::GetMethod,
        };

        let first_line = lines.get(0);

        let first_line = match first_line {
            Some(line) => line,
            None => return headers,
        };

        let (method, path) = Self::get_method_path(first_line);

        headers = HttpHeaders { method, path };

        headers
    }

    fn get_method_path(first_line: &String) -> (RequestMethod, String) {
        let slices: Vec<_> = first_line.split(' ').collect();

        let method = slices.get(0);

        let method = match method {
            Some(s) => s,
            None => return (RequestMethod::GetMethod, String::from("/")),
        };

        let method = match method.as_bytes() {
            b"GET" => RequestMethod::GetMethod,
            b"PATCH" => RequestMethod::PatchMethod,
            b"POST" => RequestMethod::PostMethod,
            b"DELETE" => RequestMethod::DeleteMethod,
            _ => RequestMethod::GetMethod,
        };

        let path = slices.get(1).unwrap_or(&"/");

        let path = path.split("?").nth(0).unwrap_or("/");

        return (method, path.to_string());
    }
}

enum RequestMethod {
    GetMethod,
    PostMethod,
    DeleteMethod,
    PutMethod,
    PatchMethod,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let http_headers = HttpHeaders::read_stream(&stream);

    let mut absolute_path = ".".to_string();

    absolute_path.push_str(&http_headers.path);

    let request_file = fs::read_to_string(&absolute_path);

    let show_file = match request_file {
        Ok(file) => file,
        Err(_) => {
            absolute_path.push_str("index.html");
            let file = fs::read_to_string(absolute_path);

            match file {
                Ok(file) => file,
                Err(_) => fs::read_to_string("404.html").unwrap(),
            }
        }
    };

    println!("path: {}", http_headers.path);

    match stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()) {
        Ok(_) => match stream.write(show_file.as_bytes()) {
            _ => return,
        },
        Err(_) => return,
    }
}
