use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
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

enum Method {
    GET
}

enum Protocol {
    V10,
    V11
}

// method and protocol are just for fun right now
// so they are not used
#[allow(dead_code)]
struct RequestHead {
    method: Method,
    uri: String,
    protocol: Protocol
}

/// Very naive and basic parser
fn get_request_head(request: &str) -> Result<RequestHead, &str> {
    let request_list: Vec<_> = request
        .split(" ")
        .collect();

    match request_list.len() {
        3 => {
            match request_list[0] {
                "GET" => {
                    let method = Method::GET;

                    let uri = request_list[1];
                    
                    // in Rust it is possible to declar a variable binding
                    // and initialize it later
                    //let protocol;
                    // if request_list[2] == "HTTP/1.0" {
                    //     protocol = Protocol::HTTP_10;
                    // } else if request_list[2] == "HTTP/1.1" {
                    //     protocol = Protocol::HTTP_11;
                    // } else { 
                    //     return Err("Unknown Protocol")
                    // };

                    // I think this is more idomatic I think, we propagate the error
                    // if no match, exactly what I wanted to do
                    let protocol = match request_list[2] {
                        "HTTP/1.0" => Protocol::V10,
                        "HTTP/1.1" => Protocol::V11,
                        _ => return Err("Unknown protocol")
                    };

                    Ok(RequestHead { method, uri: uri.to_string(), protocol })
                },
                _ => Err("Unknown method")
            }
        }
        _ => Err("Unknown route header")
    }
}

fn handle_stream(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let http_request :Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);

    match http_request.len() {
        x if x >= 1 => {
            match get_request_head(&http_request[0]) {
                // I feel the destructuring syntax weird but quite explicit
                Ok(RequestHead{uri, ..}) => {
                    let (status, file) = match uri.as_str() {
                        "/" => (Status::Ok, "hello.html"),
                        "/sleep" => {
                            thread::sleep(Duration::from_secs(5));
                            (Status::Ok, "hello.html")
                        },
                        _ => (Status::NotFound, "404.html")
                    };
        
                    let response = generate_response_content(status, file);
        
                    stream.write_all(response.as_bytes()).unwrap();
                },
                Err(message) => { 
                    println!("Get http_request_head failed with error: {}", message);
                    println!("request was:...{}...", http_request[0]);
                }
            };
        },
        _ => {}
    }
}

fn main() {
    const LISTEN_ADDRESS: &str = "0.0.0.0:80";

    let listener = TcpListener::bind(LISTEN_ADDRESS).unwrap();

    for stream in listener.incoming() {
        // iterating on incoming is like calling accept on a loop
        let active_stream = stream.unwrap();

        thread::spawn(|| {
            handle_stream(active_stream);
        });
    }
}
