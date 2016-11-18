use std::io as stdio;
use std::io::{Read, Write, BufReader, BufWriter, BufRead, Stderr};
use std::net::{TcpStream, SocketAddr, ToSocketAddrs};
use regex::Regex;
use super::status;
use super::types::{FileType, FtpError, Line, Result};

lazy_static! {
    static ref PORT_RE: Regex = Regex::new(r"\((\d+),(\d+),(\d+),(\d+),(\d+),(\d+)\)").unwrap();
    static ref SIZE_RE: Regex = Regex::new(r"\s+(\d+)\s*$").unwrap();
}

#[derive(Debug)]
pub struct FtpStream {
    bufStream: BufReader<TcpStream>,
}

impl FtpStream {
    fn write_str(&mut self, s: &str) -> Result<()> {
        if cfg!(feature = "debug_print") {
            try!(stdio::stderr().write_fmt(format_args!("CMD {}", s)));
        }
        let stream = self.bufStream.get_mut();
        
        try!(stream.write_fmt(format_args!("{}", s)));
        try!(stream.flush());
        
        Ok(())
    }

    pub fn read_response_in(&mut self, expected_codes: &[u32]) -> Result<Line> {
        let mut line = String::new();
        try!(self.bufStream.read_line(&mut line));
        if cfg!(feature = "debug_print") {
            try!(stdio::stderr().write_fmt(format_args!("FTP {}", line)));
        }
        if line.len() < 5 {
            return Err(FtpError::InvalidResponse("error: could not read reply code".to_owned()));
        }

        let code: u32 = try!(line[0..3].parse().map_err(|err| {
            FtpError::InvalidResponse(format!("error: could not parse reply code: {}", err))
        }));

        // multiple lines reply
        let expected = format!("{} ", &line[0..3]);        
        while line.len() < 5 || line[0..4] != expected {
            line.clear();
            try!(self.bufStream.read_line(&mut line));
            if cfg!(feature = "debug_print") {
                try!(stdio::stderr().write_fmt(format_args!("FTP {}", line)));
            }
        }

        if expected_codes.into_iter().any(|ec| code == *ec) {
            Ok(Line(code, line))
        } else {
            Err(FtpError::InvalidResponse(format!("Expected codes {:?} got reponse: {:?}", expected_codes, line)))
        }        
    }

    pub fn read_response(&mut self, expected_code: u32) -> Result<Line> {
        self.read_response_in(&[expected_code])
    }

    // Create a FTP Stream
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<FtpStream> {
        TcpStream::connect(addr)
            .map_err(|err| FtpError::ConnectionError(err))
            .and_then(|stream| {
                let mut ftp_stream = FtpStream{
                    bufStream: BufReader::new(stream),
                };

                ftp_stream.read_response(status::READY)
                    .map(|_| ftp_stream)
            })
    }

    
}
