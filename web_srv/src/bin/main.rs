extern crate dirs;
extern crate web_srv;
use web_srv::ThreadPool;

use std::str;
use std::path::{Path,PathBuf};
use std::io::prelude::*;
use std::net::{TcpStream,TcpListener};
use std::fs::File;


#[macro_use]
extern crate lazy_static;

lazy_static!(
    static ref DEBUG_MODE: bool = true;    //default value for debugging
    /*
    How to print a debug message sample
    println!("{}",print_debug());
    or
    if print_debug() { println!("DEBUG 0001 : {:?}",&homepath); }
    */
);

fn main() {
    let mut basepath = PathBuf::new();

    //Preparing directory path
    basepath.push(dirs::home_dir().unwrap());
    basepath.push("raspberry_pos/web_srv/html");

    if print_debug() { println!("DEBUG 0000: {:?}",&basepath); }
    
    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();
    let pool = ThreadPool::new(128);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let homepath = PathBuf::from(&basepath);
        
        pool.execute(|| {
            handle_connection(stream,homepath);
        });
    }

    println!("Shutting Down Server.Unexpected Behaviour");
}


fn handle_connection(mut stream: TcpStream,mut homepath: PathBuf) {
    let mut buffer = [0; 1500];
    
    if print_debug() { println!("DEBUG 0001 : {:?}",&homepath); }

    stream.read(&mut buffer).unwrap();
    if print_debug() { println!("DEBUG 0002 Request: {}", String::from_utf8_lossy(&buffer[..])); }

    //Split HTTP Method(GET/POST/PUT/...)
    let buffer_tmp = buffer.clone();
    //println!("Copied buffer: {}", String::from_utf8_lossy(&buffer_tmp[..])); //DEBUG
    let parsedline: Vec<&str> = str::from_utf8(&buffer_tmp).unwrap().split("\r\n").collect();
    let parsed: Vec<&str> = parsedline[0].split_whitespace().collect();
    let _parsed_len = parsed.len();
    //println!("parsed length : {}",_parsed_len); //DEBUG
    for _i in parsed.iter() {
        //println!("{}",_i); //DEBUG
    }

    let mut filename = parsed[1];

    //change requested "/" to "index.html"
    if filename == "/" {
        filename = "index.html";
    }
    else {
        filename = remove_first_slash(&filename);
        //remove first "/"
    }
    
    //println!("{}",filename); //DEBUG

    homepath.push(filename);
    println!("{:?}",homepath); //DEBUG

    //-----------------------------------------------
    //Check file requested file is found or not.
    //True if found
    //-----------------------------------------------
    //println!("{}",file_check(&homepath)); //DEBUG
    let status_line = if !file_check(&homepath) {
        homepath.set_file_name("not_found.html");
        "HTTP/1.1 404 NOT FOUND\r\n\r\n"
        //println!("File Not Found"); //DEBUG
    }
    else {
        "HTTP/1.1 200 OK\r\n\r\n"
        //println!("File Found"); //DEBUG
    };

    //-----------------------------------------------
    //Prepare requested response message
    //Read requested file
    //-----------------------------------------------
    let mut file = File::open(homepath).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

}

fn file_check(filepath: &PathBuf) -> bool {
    //println!("{}", Path::new(&filepath).exists()); //DEBUG
    Path::new(&filepath).exists()
}

fn remove_first_slash(target: &str) -> &str {
    let mut result = target.chars();
    result.next();
    result.as_str()
}

fn print_debug() -> bool{
    //println!("DEBUG : {}", *DEBUG_MODE);   //DEBUGing a DEBUG function
    if *DEBUG_MODE { true }
    else { false }
}