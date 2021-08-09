extern crate dirs;
extern crate web_srv;
use web_srv::ThreadPool;

use std::str;
use std::path::{Path,PathBuf};
use std::io::prelude::*;
use std::net::{TcpStream,TcpListener,Shutdown};
use std::fs::File;
use std::sync::Mutex;
use std::{thread, time};
use std::time::Duration;

//----------DEFAULT PARAMETER----------//
const MAX_POOL_THREAD: usize = 8;
const BUFFER: usize = 8000;     //request read buffer
const DEBUG: bool = false;       //debug configuration

#[macro_use]
extern crate lazy_static;

lazy_static!(
    static ref DEBUG_MODE: Mutex<bool> = Mutex::new(DEBUG);
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
    //TODO Read configuration file to change debug configuration
    change_debug(true);

    //Preparing directory path
    basepath.push(dirs::home_dir().unwrap());
    basepath.push("raspberry_pos/web_srv/html");

    if print_debug() { println!("DEBUG 0000 : {:?}",&basepath); }
    
    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();
    let pool = ThreadPool::new(MAX_POOL_THREAD);

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
    let mut buffer = [0; BUFFER]; //Read buffer
    
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
    else {
        filename = "/noresource.html"
    }

    //change requested "/" to "index.html"
    if filename == "/" {
        filename = "index.html";
    }
    else {
        //remove first "/"
        filename = remove_first_slash(&filename);
    }

    //reparse request URI
    //example: /aaa/bbb/ccc.html -> {"","aaa","bbb","ccc.html"}
    let mut parsed_uri: Vec<&str> = parsed[1].split("/").collect();
    //remove first null vector -> {"aaa","bbb","ccc.html"}
    parsed_uri.drain(0..1);
    let _parsed_uri_len = parsed_uri.len();
    if print_debug() { println!("DEBUG 0020 : parsed_uri length : {}",_parsed_uri_len); }
    for _i in parsed_uri.iter() {
        if print_debug() { println!("DEBUG 0019 : {}",_i); }
    }

    //Recognizing requested resource or data format
    //aaa/bbb/ccc.html -> "html" file
    //To process data format
    let mut data_format: Vec<&str> = parsed_uri[_parsed_uri_len - 1].split(".").collect();
    let resource_format = match data_format.len() {
        //not a file or no dot on request
        1 => {
            if print_debug() { println!("DEBUG 0026 : requested resource is {}",data_format[0]); }
            "NONE"
        },
        2 => {
            if print_debug() { println!("DEBUG 0027 : requested resource format is {}",data_format[1]); }
            data_format[1]
        },
        _ => {
            if print_debug() { println!("DEBUG 0028 : unable to recognize requested resource format"); }
            "UNKNOWN"
        },
    };

    //let mut _response = String::new();
    let mut _response: Vec<u8> = Vec::new();
    let mut lastresort_request: bool = false;

    //Evaluate API Call
    if print_debug() { println!("DEBUG 0013 : is api call {}",is_api(&filename)); }

    //Process branch for API Call
    if is_api(&filename) {
        if print_debug() { println!("DEBUG 0016 : is api call {}",is_api(&filename)); }
        //Evaluate Request Method
        match parsed[0] {
            "GET" => { 
                if _parsed_uri_len > 1 {
                    if print_debug() { println!("DEBUG 0023 : Matching api resource"); }
                    match parsed_uri[1] {
                        "test_sse.rs" => get_api_testsse(stream,&homepath,&filename),
                        _ => get_api_lastresort(stream,&homepath,&filename),
                    }
                }
                else {
                    get_api_lastresort(stream,&homepath,&filename);
                }
            },
            "POST" => api_lastresort(stream,&homepath,&filename),
            "PUT" => api_lastresort(stream,&homepath,&filename),
            _ => api_lastresort(stream,&homepath,&filename),
        }
    }

    //Process branch for !API Call
    else{
        //Evaluate Request Method  
        match parsed[0] {
            "GET" => {
                match resource_format {
                    "html" => _response = get_request(&homepath,&filename).into_bytes(),
                    "jpg" => _response = get_jpg_request(&homepath,&filename),
                    "NONE" => _response = format!("NOT IMPLEMENTED").into_bytes(),
                    "UNKNOWN" => _response = format!("NOT IMPLEMENTED").into_bytes(),
                    _ => _response = format!("NOT IMPLEMENTED").into_bytes(),
                }
            },
            "POST" => _response = format!("NOT IMPLEMENTED").into_bytes(),
            "PUT" => _response = format!("NOT IMPLEMENTED").into_bytes(),
            _ => {
                lastresort_request = true;
                _response = format!("UNDEFINED REQUEST METHOD").into_bytes()
            }
        }

        if lastresort_request {
            stream.shutdown(Shutdown::Both).expect("shutdown call failed");
            if print_debug() { println!("DEBUG 0018 : REQUEST GOES TO LAST RESORT"); }
        }
        else {
            //stream.write(_response.as_bytes()).unwrap();
            stream.write(&_response).unwrap();
            stream.flush().unwrap();
        }
    }

}


//----------BEGIN OF REQUEST ROUTING FUNCTION---------//

//-----------------------------------------------
//Funtion To Process GET Method
//Retrieve homedir as homepath and
//Request URI as filename
//-----------------------------------------------
fn get_request<'a>(homepath: &'a PathBuf,filename: &str) -> String {
    let mut filepath = homepath.clone();
    //Merge homedir with requested URI to point requested file
    if print_debug() { println!("DEBUG 0007 : Filename {}",filename); }
    filepath.push(filename);
    if print_debug() { println!("DEBUG 0008 : Homepath {:?}",filepath); }

    //-----------------------------------------------
    //Check file requested file is found or not.
    //True if found
    //-----------------------------------------------
    if print_debug() { println!("DEBUG 0009 : is file found? {}",file_check(&filepath)); }
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

fn get_jpg_request(homepath: &PathBuf,filename: &str) -> Vec<u8> {
    let mut filepath = homepath.clone();
    //Merge homedir with requested URI to point requested file
    if print_debug() { println!("DEBUG 0029 : Filename {}",filename); }
    filepath.push(filename);
    if print_debug() { println!("DEBUG 0030 : Homepath {:?}",filepath); }

    //-----------------------------------------------
    //Check file requested file is found or not.
    //True if found
    //-----------------------------------------------
    if print_debug() { println!("DEBUG 0031 : is file found? {}",file_check(&filepath)); }
    let (status_line,flag) = if !file_check(&filepath) {
            //filepath.set_file_name("not_found.html");
            filepath = homepath.clone();
            filepath.push("not_found.html");
            if print_debug() { println!("DEBUG 0032 : File Not Found"); }
            ("HTTP/1.1 404 NOT FOUND\r\n\r\n",0)
        }
        else {
            if print_debug() { println!("DEBUG 0033 : File Found"); }
            ("HTTP/1.1 200 OK\r\n\r\n",1)
        };
    
    //-----------------------------------------------
    //Prepare requested response message
    //Read requested file
    //-----------------------------------------------
    
    if print_debug() { println!("DEBUG 0034 : {:?}",filepath); }
    let mut file = File::open(filepath).unwrap();
    let mut contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut contents).unwrap();

    [status_line.as_bytes().to_vec() , contents].concat()
}

//NOT IMPLEMENTED YET
//-----------------------------------------------
//Funtion To Process POST Method
//Retrieve homedir as homepath and
//Request URI as filename
//-----------------------------------------------
// fn post_request(homepath: &PathBuf,filename: &str) -> &str {}



//NOT IMPLEMENTED YET
//-----------------------------------------------
//Funtion To Process PUT Method
//Retrieve homedir as homepath and
//Request URI as filename
//-----------------------------------------------
// fn put_request(homepath: &PathBuf,filename: &str) -> &str {}



//-----------------------------------------------
//Funtion To Process GET API Call
//Retrieve homedir as homepath and
//Request URI as resources and
//Stream as TCPStream
//-----------------------------------------------
fn get_api_testsse(mut stream: TcpStream,homepath: &PathBuf,filename: &str) {
    if print_debug() { println!("DEBUG 0022 : \"TESTSSE\" GET API CALLED"); }
    let mut response = String::new();
    let mut contents = String::new();
    let mut buffer = [0; BUFFER];

    //Prepare HTTP Header
    let status_line = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\n\r\n";

    //stream.shutdown(Shutdown::Read).expect("shutdown call failed");
    loop {
        //contents = "data: {\"data\": \"test content\"}\r\n\r\n".to_string();
        contents = "data: <img src=\"image/test.jpg\" alt=\"sample\">\r\n\r\n".to_string();

        response = format!("{}{}", status_line, contents);

        if print_debug() { println!("DEBUG 0024 : Response : {:?}",response); }
        
        //stream.write(response.as_bytes()).unwrap();
        //escaping from possible write on rst socket
        //exit thread before panic
        match stream.write(response.as_bytes()) {
            Ok(n) => stream.flush().unwrap(),
            Err(err) => break,
        }

        thread::sleep(time::Duration::from_millis(10000));
    }
}


fn get_api_lastresort(mut stream: TcpStream,homepath: &PathBuf,filename: &str) {
    if print_debug() { println!("DEBUG 0021 : GET API CALL GOES TO LAST RESORT"); }
    let mut _response = String::new();
    _response = format!("NOT IMPLEMENTED");
    stream.write(_response.as_bytes()).unwrap();
    stream.flush().unwrap();
}


//-----------------------------------------------
//Funtion To Process UNKNOWN API Call
//Retrieve homedir as homepath and
//Request URI as resources and
//Stream as TCPStream
//Shutdown TCP Connection immediately
//-----------------------------------------------
fn api_lastresort(mut _stream: TcpStream,homepath: &PathBuf,filename: &str) {
    _stream.shutdown(Shutdown::Both).expect("shutdown call failed");
    if print_debug() { println!("DEBUG 0017 : API CALL GOES TO LAST RESORT"); }
}


//-----------END OF REQUEST ROUTING FUNCTION----------//


fn file_check(filepath: &PathBuf) -> bool {
    if print_debug() { println!("DEBUG 0025 : is file exist? {}", Path::new(&filepath).exists()); }
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