use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn handle_stream(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let http_request :Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);

    let status_line = "HTTP/1.1 200 OK";
    
    let content = fs::read_to_string("hello.html").unwrap();

    let content_length = content.len();

    let response = 
        format!("{status_line}\r\nContent-Length:{content_length}\r\n\r\n{content}");

    stream.write_all(response.as_bytes()).unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:80").unwrap();

    for stream in listener.incoming() {
        // iterating on incoming is like calling accept on a loop
        let active_stream = stream.unwrap();

        handle_stream(active_stream);
    }
}
