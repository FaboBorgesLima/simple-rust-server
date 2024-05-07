use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

pub struct HttpHeader {
    pub path: String,
    pub method: RequestMethod,
}

pub enum RequestMethod {
    GetMethod,
    PostMethod,
    DeleteMethod,
    PutMethod,
    PatchMethod,
}

impl HttpHeader {
    pub fn read_stream(mut stream: &TcpStream) -> Self {
        let buf_reader = BufReader::new(&mut stream);

        let lines: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let mut headers = HttpHeader {
            path: String::new(),
            method: RequestMethod::GetMethod,
        };

        let first_line = lines.get(0);

        let first_line = match first_line {
            Some(line) => line,
            None => return headers,
        };

        let (method, path) = Self::get_method_path(first_line);

        headers = HttpHeader { method, path };

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
            b"PUT" => RequestMethod::PutMethod,
            _ => RequestMethod::GetMethod,
        };

        let path = slices.get(1).unwrap_or(&"/");

        let path = path.split("?").nth(0).unwrap_or("/");

        return (method, path.to_string());
    }
}
