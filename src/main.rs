extern crate FTPCLI;

use FTPCLI::FtpStream;
use FTPCLI::FtpError;

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

    // ftpStream.rm("test.txt").unwrap();
    ftpStream.quit().unwrap();
}
