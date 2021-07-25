extern crate dirs;
extern crate web_srv;
use web_srv::ThreadPool;

use std::thread;
use std::str;
use std::time::Duration;
use std::path::PathBuf;
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::fs::File;
use std::path::Path;

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
    let (status_line,mut filename) = if buffer.starts_with(req_get_none) {
        //("HTTP/1.1 200 OK\r\n\r\n", "/home/ryo/web_srv/html/mainpage.html")
        ("HTTP/1.1 200 OK\r\n\r\n", "mainpage.html")
    }
    else {
        ("HTTP/1.1 200 OK\r\n\r\n", "not_implemented.html")
    };

    //Split HTTP Method(GET/POST/PUT/...)
    let buffer_tmp = buffer.clone();
    println!("Copied buffer: {}", String::from_utf8_lossy(&buffer_tmp[..]));
    //let mut split = buffer_tmp.as_str().split_whitespace();
    //let mut split = str::from_utf8(&buffer_tmp).unwrap().split_whitespace();
    //println!("{:?}",split.next());
    //let parsed: Vec<&str> = str::from_utf8(&buffer_tmp).unwrap().split_whitespace().collect();
    let parsedline: Vec<&str> = str::from_utf8(&buffer_tmp).unwrap().split("\r\n").collect();
    let parsed: Vec<&str> = parsedline[0].split_whitespace().collect();
    let parsed_len = parsed.len();
    println!("parsed length : {}",parsed_len);
    for i in parsed.iter() {
        println!("{}",i);
    }

    filename = parsed[1];

    //change requested "/" to "index.html"
    if filename == "/" {
        filename = "index.html";
    }
    else {
        filename = remove_first_slash(&filename);
        //remove first "/"
    }
    
    //println!("{}",filename);

    homepath.push(filename);
    println!("{:?}",homepath);

    //Check file requested file is found or not.
    //True if found
    //println!("{}",file_check(&homepath));
    if !file_check(&homepath){
        homepath.set_file_name("not_found.html");
        println!("File Not Found");
    }
    else {
        println!("File Found");
    }

    let mut file = File::open(homepath).unwrap();


    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

}

fn file_check(filepath: &PathBuf) -> bool {
    println!("{}", Path::new(&filepath).exists());
    Path::new(&filepath).exists()
}

fn remove_first_slash(target: &str) -> &str {
    let mut result = target.chars();
    result.next();
    result.as_str()
}