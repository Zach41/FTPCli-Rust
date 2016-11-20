extern crate FTPCLI;
extern crate chrono;

use std::fs::File;
use std::io::Write;
use std::io::Read;
use chrono::{DateTime, UTC, TimeZone, Timelike, Datelike};

use FTPCLI::FtpStream;
use FTPCLI::FtpError;
use FTPCLI::status;

fn main() {
    let mut ftpStream = FtpStream::connect("182.254.245.238:21").unwrap();

    ftpStream.login("zach", "admin123123").unwrap();

    // ftpStream.cwd("~/hello").unwrap();

    // ftpStream.cdup().unwrap();

    ftpStream.pwd().map(|pwd| println!("{}", pwd));

    ftpStream.noop().unwrap();

    ftpStream.mkdri("hello2");
    
    ftpStream.pasv().unwrap();

    ftpStream.rename("hello2", "hello");

    // ftpStream.rmdir("hello").unwrap();

    ftpStream.size("demo.c").unwrap();
    ftpStream.size("hello.txt").map(|ret| {
        match ret {
            Some(size) => println!("FILE SIZE: {}", size),
            None => println!("FILE NOT EXISTS"),
        }
    });
    ftpStream.retr("sig_recv.c", |stream| {
        let mut file = File::create("sig_recv.c").unwrap();
        let mut buf = [0; 2048];

        loop {
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    file.write_all(&mut buf[0 .. n]).unwrap()
                },
                Err(err) => return Err(FtpError::ConnectionError(err))
            };
        }

        Ok(())
    });

    let mut reader = ftpStream.get("sig_recv.c").unwrap();
    let mut file = File::create("sig_recv2.c").unwrap();
    let mut buf = [0; 2048];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => file.write_all(&mut buf[0 .. n]).unwrap(),
            Err(err) => println!("{:?}", err),
        }
    }
    // have to read response
    ftpStream.read_response(status::CLOSING_DATA_CONNECTION).unwrap();
    drop(reader);
    // ftpStream.rm("test.txt").unwrap();

    let lines = ftpStream.list(None).unwrap();
    for line in lines {
        println!("{}", line);
    }

    let lines = ftpStream.nlist(None).unwrap();
    for line in lines {
        println!("{}", line);
    }

    let datetime = ftpStream.mdtm("sig_recv.c").unwrap();
    match datetime {
        Some(time) => {
            println!("{}.{}.{} {}:{}", time.year(), time.month(), time.day(), time.hour(), time.minute());
        },
        None => {
            println!("Could not get file modification time");
        }
    }
    ftpStream.quit().unwrap();
}
