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
    if print_debug() { println!("DEBUG 0004 : Copied buffer: {}", String::from_utf8_lossy(&buffer_tmp[..])); }
    let parsedline: Vec<&str> = str::from_utf8(&buffer_tmp).unwrap().split("\r\n").collect();
    let parsed: Vec<&str> = parsedline[0].split_whitespace().collect();
    let _parsed_len = parsed.len();
    if print_debug() { println!("DEBUG 0005 : parsed length : {}",_parsed_len); }
    for _i in parsed.iter() {
        if print_debug() { println!("DEBUG 0006 : {}",_i); }
    }

    let mut filename = parsed[1];

    

    //change requested "/" to "index.html"
    if filename == "/" {
        filename = "index.html";
    }
    else {
        filename = remove_first_slash(&filename);
        //remove first "/"
        if print_debug() { println!("DEBUG 0013 : is api call {}",is_api(&filename)); }

        //-----------------------------------------------
        //Process For API Call
        //path = /api/xxx
        //-----------------------------------------------
        if is_api(&filename) {
            if print_debug() { println!("DEBUG 0013 : is api call {}",is_api(&filename)); }
        }
        //-----------------------------------------------
        //Process For !API Call
        //-----------------------------------------------
        else{
            //TO DO GET METHOD or POST METHOD


            //-----------------------------------------------
            //Process HTTP GET Request
            //-----------------------------------------------
            if print_debug() { println!("DEBUG 0007 : Filename {}",filename); }

            homepath.push(filename);
            if print_debug() { println!("DEBUG 0008 : Homepath {:?}",homepath); }

            //-----------------------------------------------
            //Check file requested file is found or not.
            //True if found
            //-----------------------------------------------
            if print_debug() { println!("DEBUG 0009 : {}",file_check(&homepath)); }

            let (status_line,found_flag) = if !file_check(&homepath) {
                homepath.set_file_name("not_found.html");
                if print_debug() { println!("DEBUG 0010 : File Not Found"); }
                ("HTTP/1.1 404 NOT FOUND\r\n\r\n",0)
                }
                else {
                    if print_debug() { println!("DEBUG 0011 : File Found"); }
                    ("HTTP/1.1 200 OK\r\n\r\n",1)
                };

            //-----------------------------------------------
            //Prepare requested response message
            //Read requested file
            //-----------------------------------------------
            if found_flag == 1 {
                let mut file = File::open(homepath).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();

                let response = format!("{}{}", status_line, contents);

                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
        }
        
    }
    
    

    

}

fn file_check(filepath: &PathBuf) -> bool {
    if print_debug() { println!("{}", Path::new(&filepath).exists()); }
    Path::new(&filepath).exists()
}

fn remove_first_slash(target: &str) -> &str {
    let mut result = target.chars();
    result.next();
    result.as_str()
}

fn is_api(requests: &str) -> bool {
    let parsedrequests: Vec<&str> = requests.split("/").collect();
    for _i in parsedrequests.iter() {
        if print_debug() { println!("DEBUG 0012 : {}",_i); }
    }
    if parsedrequests[0] == "api" {
        true
    }
    else {
        false
    }
}

fn print_debug() -> bool{
    //println!("DEBUG : {}", *DEBUG_MODE);   //DEBUGing a DEBUG function
    if *DEBUG_MODE { true }
    else { false }
}