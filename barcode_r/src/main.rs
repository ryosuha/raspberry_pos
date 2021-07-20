#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!("input.rs");

use std::io;
use std::io::Read;
use std::fs::File;

fn read_input_event(f: &mut File) -> io::Result<input_event> {
    const sz: usize = std::mem::size_of::<input_event>();
    let mut buf = [0; sz];
    f.read_exact(&mut buf)?;
    let ie: input_event = unsafe { std::mem::transmute(buf) };
    Ok(ie)
}

fn dump_input_event(fname: &str) {
    let mut _cnt = 0;        //To prevent double count of barcode reader always read number twice.
    let mut _f0 = 0;         //To remove first double "0".barcode reader always begin with double 0.
    let mut _e0 = 0;         //To remove last double "0".barcode reader always begin with double 0.
    let mut _tmp:i32 = 0;
    let mut s = String::new(); 
    //println!("{}", s);
    let mut f = File::open(fname).expect("open failed.");
    loop {
        match read_input_event(&mut f).expect("read error.") {
            input_event { type_, code,value, .. } if type_ as u32 == 4 => {
                match code as u32 {
                    4 => {
                        if value < 0 {
                            println!("less than 0 error");
                        }
                        // misread the specification.
                        // commented out reason is :
                        // dont't needed to remove first double "0" input.
                        /*
                        else if value == 458791 && _f0 <= 1 {
                            if _f0 == 0 {
                                println!("START");
                                _f0 += 1;
                            }
                            else{
                                println!("Second START");
                                _f0 += 1;
                            }
                        }
                        */
                        else if value == 458792 && _e0 <= 1 {
                            if _e0 == 0 {
                                println!("READED = {}",s);
                                s.clear();
                                _e0 += 1;
                            }
                            else{
                                //Initialize barcode read iteration.
                                _e0 = 0;
                                _f0 = 0;
                                //println!("END READ");
                                break;
                            }
                        }
                        else if _cnt == 0{
                            _cnt += 1;
                            println!("{:?}",((value - 458781) % 10) );
                            _tmp = ( value - 458781 ) % 10 ;
                            s.push_str(&_tmp.to_string());
                            
                        }
                        else if _cnt == 1{
                            _cnt -= 1;
                        }
                    }
                    _ => {}
                }
            }
            //FOR DEBUG PURPOSE
            //ie => println!("input_event ={:?}", ie),
            _ => {}
        }        
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} /dev/input/event0", &args[0]);
        return;
    }

    loop {
        dump_input_event(&args[1]);
    }

}
