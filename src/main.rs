extern crate FTPCLI;
extern crate chrono;
extern crate rpassword;

use std::io as stdio;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::process::exit;
use std::env;
use std::net::{ToSocketAddrs};
use chrono::{Timelike, Datelike};
use rpassword::prompt_password_stdout;

use FTPCLI::FtpStream;
use FTPCLI::FtpError;
use FTPCLI::status;

fn cmd_loop() -> Vec<String> {
    println!("ftp> ");
    
    let mut cmd_line = String::new();
    stdio::stdin().read_to_string(&mut cmd_line).unwrap();

    cmd_line.split(' ').into_iter()
        .map(|s| String::from(s))
        .filter(|s| s.len() > 0).collect()
}

fn login(ftp_stream: &mut FtpStream) {
    print!("Name: ");
    let mut name: String = String::new();
    // let mut passwd: String = String::new();

    println!("Name: ");
    stdio::stdin().read_line(&mut name).unwrap();
    let passwd = prompt_password_stdout("Password: ").unwrap();

    match ftp_stream.login(&name, &passwd) {
        Ok(()) => println!("Login successfully"),
        Err(err) => println!("Login failed: {}", err.to_string()),
    }
}

fn connect(ip: &str, port: u32) -> Option<FtpStream> {
    let addr = format!("{}:{}", ip, port);
    match FtpStream::connect(&addr[..]) {
        Ok(stream) => Some(stream),
        Err(err) => {
            println!("Connection Failed: {}", err.to_string());
            None
        }
    }
}

fn main() {
}
