extern crate dirs;
extern crate web_srv;
use web_srv::ThreadPool;

use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::fs::File;

fn main() {
    let mut basepath = PathBuf::new();
    basepath.push(dirs::home_dir().unwrap());
    basepath.push("raspberry_pos/web_srv/html");
    println!("{:?}",&basepath);

    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();
    let pool = ThreadPool::new(12);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let homepath = PathBuf::from(&basepath);

        pool.execute(|| {
            handle_connection(stream,homepath);
        });
    }

    println!("Shutting Down Server");
}


fn handle_connection(mut stream: TcpStream,mut homepath: PathBuf) {
    let mut buffer = [0; 1500];
    println!("DEBUG 0000 : {:?}",&homepath);

    stream.read(&mut buffer).unwrap();
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    //Request Type
    let req_get_none = b"GET / HTTP/1.1\r\n";

    //Prepare Response File
    let (status_line, filename) = if buffer.starts_with(req_get_none) {
        //("HTTP/1.1 200 OK\r\n\r\n", "/home/ryo/web_srv/html/mainpage.html")
        ("HTTP/1.1 200 OK\r\n\r\n", "mainpage.html")
    }
    else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "not_found.html")
    };

    homepath.push(filename);
    println!("{:?}",homepath);
    let mut file = File::open(homepath).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    /*
    if buffer.starts_with(req_get_none) {
        homepath.push("mainpage.html");
        let mut file = File::open(homepath).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let response = format!("HTTP/1.1 200 OK\r\n\r\n{}",contents);
        println!("Response: {}", &response);

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
    else {
        let response = format!("HTTP/1.1 404 OK\r\n\r\n");

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
    */   

    
}