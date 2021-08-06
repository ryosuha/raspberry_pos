extern crate dirs;
extern crate web_srv;
use web_srv::ThreadPool;

use std::str;
use std::path::{Path,PathBuf};
use std::io::prelude::*;
use std::net::{TcpStream,TcpListener,Shutdown};
use std::fs::File;
use std::sync::Mutex;

#[macro_use]
extern crate lazy_static;

lazy_static!(
    static ref DEBUG_MODE: Mutex<bool> = Mutex::new(false);    //default value for debugging
    /*
    How to print a debug message sample
    println!("{}",print_debug());
    or
    if print_debug() { println!("DEBUG 0001 : {:?}",&homepath); }
    */
);


fn main() {
    let mut basepath = PathBuf::new();

    //test changing debug mode change_debug();
    change_debug(true);

    //Preparing directory path
    basepath.push(dirs::home_dir().unwrap());
    basepath.push("raspberry_pos/web_srv/html");

    if print_debug() { println!("DEBUG 0000 : {:?}",&basepath); }
    
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


fn handle_connection(mut stream: TcpStream,homepath: PathBuf) {
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

    let mut filename = "/";
    if _parsed_len > 1 {
        filename = parsed[1];
    }

    //change requested "/" to "index.html"
    if filename == "/" {
        filename = "index.html";
    }
    else {
        //remove first "/"
        filename = remove_first_slash(&filename);
    }

    let mut _response = String::new();
    let mut lastresort_request: bool = false;

    //Evaluate API Call
    if print_debug() { println!("DEBUG 0013 : is api call {}",is_api(&filename)); }

    //Process branch for API Call
    if is_api(&filename) {
        if print_debug() { println!("DEBUG 0016 : is api call {}",is_api(&filename)); }
        //Evaluate Request Method
        match parsed[0] {
            "GET" => get_api(stream,&homepath,&filename),
            "POST" => lastresort_api(stream,&homepath,&filename),
            "PUT" => lastresort_api(stream,&homepath,&filename),
            _ => lastresort_api(stream,&homepath,&filename),
        }
    }

    //Process branch for !API Call
    else{
        //Evaluate Request Method  
        match parsed[0] {
            "GET" => _response = response_http_get(&homepath,&filename),
            "POST" => _response = format!("NOT IMPLEMENTED"),
            "PUT" => _response = format!("NOT IMPLEMENTED"),
            _ => {
                lastresort_request = true;
                _response = format!("UNDEFINED REQUEST METHOD")
            }
        }

        if lastresort_request {
            stream.shutdown(Shutdown::Both).expect("shutdown call failed");
            if print_debug() { println!("DEBUG 0018 : REQUEST GOES TO LAST RESORT"); }
        }
        else {
            stream.write(_response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }

}

//-----------------------------------------------
//Funtion To Process GET Method
//Retrieve homedir as homepath and
//Request URI as filename
//-----------------------------------------------
fn response_http_get<'a>(homepath: &'a PathBuf,filename: &str) -> String {
    let mut filepath = homepath.clone();
    //Merge homedir with requested URI to point requested file
    if print_debug() { println!("DEBUG 0007 : Filename {}",filename); }
    filepath.push(filename);
    if print_debug() { println!("DEBUG 0008 : Homepath {:?}",filepath); }

    //-----------------------------------------------
    //Check file requested file is found or not.
    //True if found
    //-----------------------------------------------
    if print_debug() { println!("DEBUG 0009 : {}",file_check(&filepath)); }
    let status_line = if !file_check(&filepath) {
            //filepath.set_file_name("not_found.html");
            filepath = homepath.clone();
            filepath.push("not_found.html");
            if print_debug() { println!("DEBUG 0010 : File Not Found"); }
            "HTTP/1.1 404 NOT FOUND\r\n\r\n"
        }
        else {
            if print_debug() { println!("DEBUG 0011 : File Found"); }
            "HTTP/1.1 200 OK\r\n\r\n"
        };
    
    //-----------------------------------------------
    //Prepare requested response message
    //Read requested file
    //-----------------------------------------------
    
    if print_debug() { println!("DEBUG 0015 : {:?}",filepath); }
    let mut file = File::open(filepath).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    format!("{}{}", status_line, contents)

}

//NOT IMPLEMENTED YET
//-----------------------------------------------
//Funtion To Process POST Method
//Retrieve homedir as homepath and
//Request URI as filename
//-----------------------------------------------
// fn post_request(mut stream: TcpStream,homepath: &PathBuf,filename: &str) -> &str {}

//NOT IMPLEMENTED YET
//-----------------------------------------------
//Funtion To Process PUT Method
//Retrieve homedir as homepath and
//Request URI as filename
//-----------------------------------------------
// fn put_request(mut stream: TcpStream,homepath: &PathBuf,filename: &str) -> &str {}

//-----------------------------------------------
//Funtion To Process GET API Call
//Retrieve homedir as homepath and
//Request URI as resources and
//Stream as TCPStream
//-----------------------------------------------
fn get_api(mut stream: TcpStream,homepath: &PathBuf,filename: &str) {
    let mut _response = String::new();
    _response = format!("NOT IMPLEMENTED");
    stream.write(_response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn lastresort_api(mut stream: TcpStream,homepath: &PathBuf,filename: &str) {
    //let mut _response = String::new();
    //_response = format!("NOT IMPLEMENTED");
    //stream.write(_response.as_bytes()).unwrap();
    //stream.flush().unwrap();
    stream.shutdown(Shutdown::Both).expect("shutdown call failed");
    if print_debug() { println!("DEBUG 0017 : API CALL GOES TO LAST RESORT"); }
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

fn change_debug(flag: bool) {
    if flag { *DEBUG_MODE.lock().unwrap() = true; }
    else { *DEBUG_MODE.lock().unwrap() = false; }
}

fn print_debug() -> bool{
    //println!("DEBUG : {}", *DEBUG_MODE);   //DEBUGing a DEBUG function
    //if *DEBUG_MODE { true }
    if *DEBUG_MODE.lock().unwrap() { true }
    else { false }
}