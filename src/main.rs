use std::{
    io::{prelude::*, BufReader},
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
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    println!("Request: {:#?}", http_request);

    let first_line = http_request.get(0).unwrap();

    match stream.write_all(get_path(first_line).unwrap().as_bytes()) {
        _ => return,
    }
}

fn get_path(first_line: &String) -> Option<String> {
    let parts: Vec<String> = first_line
        .split(&[' ', '?'][..])
        .map(|s| String::from(s))
        .collect();

    let path = match parts.get(1) {
        Some(path) => Some(String::from(path)),
        None => return None,
    };

    path
}
