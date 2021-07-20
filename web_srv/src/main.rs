extern crate dirs;

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
    basepath.push("web_srv/html");
    println!("{:?}",&basepath);

    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let homepath = PathBuf::from(&basepath);

        thread::spawn(|| {
            handle_connection(stream,homepath);
        });
    }
}


fn handle_connection(mut stream: TcpStream,mut homepath: PathBuf) {
    let mut buffer = [0; 1500];
    println!("DEBUG 0000 : {:?}",&homepath);

    stream.read(&mut buffer).unwrap();
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    //Prepare Response File
    homepath.push("mainpage.html");
    let mut file = File::open(homepath).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}",contents);
    println!("Response: {}", &response);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}