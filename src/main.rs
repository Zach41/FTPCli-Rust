extern crate FTPCLI;

use FTPCLI::FtpStream;

fn main() {
    let mut ftpStream = FtpStream::connect("127.0.0.1:21");
}
