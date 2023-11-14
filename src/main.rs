use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
    sync::{mpsc, Mutex, Arc}
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

// TODO: is trait object needed or could I just use function pointers ?
type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    //n_threads: usize,
    sender: mpsc::Sender<Job>
}

impl ThreadPool {
    fn new(n_threads: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel::<Job>();
        
        let shared_mutex_receiver = Arc::new(Mutex::new(receiver));

        for i in 0..n_threads {
            // MPSC: Multiple Producer, Single Receiver
            // So we have to share the receiver, we use the patter of the book: Arc of a Mutex

            let rx_mutex = shared_mutex_receiver.clone();
            
            thread::spawn(move || {
                loop {
                    // receive is blocking so the thread will wait for a Job
                    // Note: I think this could be a problem as the following code
                    // won't work as expected:
                    // let rx = rx_mutex.lock().unwrap();
                    // let f = rx.recv().unwrap();
                    // because of the lifetime of rx (MutexGuard), the lock will be kept until the
                    // end of the scope (so after f is executed) and rx is destroyed
                    // so we have to do something like this to benefit from the fact that the temporaries
                    // on the rhs are dropped after the let statement
                    // TODO: what if we want to do something else than unwrap ?
                    let f = rx_mutex.lock().unwrap().recv().unwrap();
                    println!("Worker: {} received a Job, executing it", i);
                    f();
                }
            });
        }

        ThreadPool { sender }
    }

    fn execute<F>(&self, f: F) 
    where F: FnOnce() + Send + 'static {
        self.sender.send(Box::new(f)).unwrap();
    }

}

fn main() {
    const LISTEN_ADDRESS: &str = "0.0.0.0:80";

    let listener = TcpListener::bind(LISTEN_ADDRESS).unwrap();

    let thread_pool = ThreadPool::new(5);

    for stream in listener.incoming() {
        // iterating on incoming is like calling accept on a loop
        let active_stream = stream.unwrap();

        thread_pool.execute(|| handle_stream(active_stream));

        // thread::spawn(|| {
        //     handle_stream(active_stream);
        // });
    }
}
