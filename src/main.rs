use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

enum Status {
    Ok,
    NotFound
}

fn generate_response_content(status: Status, file: &str) -> String {
    let status_line = match status {
        Status::Ok => "HTTP/1.1 200 OK",
        _ => "HTTP/1.1 404 NOT FOUND"
    };

    let content = fs::read_to_string(file).unwrap();

    let content_length = content.len();

    let response = 
        format!("{status_line}\r\nContent-Length:{content_length}\r\n\r\n{content}");

    response
}

fn handle_stream(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let http_request :Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);

    let (status, file) = match http_request[0].as_str() {
        "GET / HTTP/1.1" => (Status::Ok, "hello.html"),
        _ => (Status::NotFound, "404.html")
    };

    let response = generate_response_content(status, file);

    stream.write_all(response.as_bytes()).unwrap();
}

fn main() {
    const LISTEN_ADDRESS: &str = "127.0.0.1:80";

    let listener = TcpListener::bind(LISTEN_ADDRESS).unwrap();

    for stream in listener.incoming() {
        // iterating on incoming is like calling accept on a loop
        let active_stream = stream.unwrap();

        handle_stream(active_stream);
    }
}
