mod http_header;

use http_header::RequestMethod;

use crate::http_header::HttpHeader;

use std::{
    fs,
    io::Write,
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let http_headers = HttpHeader::read_stream(&stream);

    let absolute_path = get_absolute_path(&http_headers.path);

    let request_file = fs::read_to_string(&absolute_path);

    println!("request path: {}", http_headers.path);
    println!("file path {}", absolute_path);

    println!("method {}", method_to_string(&http_headers.method));

    let show_file = match request_file {
        Ok(file) => file,
        Err(_) => {
            let page404 = fs::read_to_string("./404.html");

            match page404 {
                Ok(page404) => page404,
                Err(_) => "add 404 page".to_string(),
            }
        }
    };

    match stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()) {
        Ok(_) => match stream.write(show_file.as_bytes()) {
            _ => return,
        },
        Err(_) => return,
    }
}

fn get_absolute_path(path: &String) -> String {
    let mut absolute_path = String::from(".");

    absolute_path.push_str(path);

    if !path.contains('.') {
        if path.ends_with('/') {
            absolute_path.push_str("index.html");
            return absolute_path;
        }
        absolute_path.push_str("/index.html");

        return absolute_path;
    };

    return absolute_path;
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
