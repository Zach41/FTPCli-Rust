use std::io as stdio;
use std::io::{Read, Write, BufReader, BufWriter, BufRead, Stderr};
use std::net::{TcpStream, SocketAddr, ToSocketAddrs};
use std::str::FromStr;
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

    /// Log into the FTP server
    pub fn login(&mut self, username: &str, passwd: &str) -> Result<()> {
        let username_cmd = format!("USER {}\r\n", username);
        try!(self.write_str(&username_cmd));

        self.read_response_in(&[status::LOGGED_IN, status::NEED_PASSWORD])
            .and_then(|Line(code, _)| {
                if code == status::NEED_PASSWORD {
                    let passwd_cmd = format!("PASS {}\r\n", passwd);
                    try!(self.write_str(&passwd_cmd));
                    try!(self.read_response(status::LOGGED_IN));
                }
                Ok(())
            })
    }

    /// Change the current working directory
    pub fn cwd(&mut self, path: &str) -> Result<()> {
        let cwd_cmd = format!("CWD {}\r\n", path);
        try!(self.write_str(&cwd_cmd));
        self.read_response(status::REQUESTED_FILE_ACTION_OK).map(|_| ())
    }

    /// Move to the parent directory
    pub fn cdup(&mut self) -> Result<()> {
        let cdup_cmd = format!("CDUP\r\n");
        try!(self.write_str(&cdup_cmd));
        self.read_response(status::REQUESTED_FILE_ACTION_OK).map(|_| ())
    }

    /// Get the current working directory
    pub fn pwd(&mut self) -> Result<String> {
        let pwd_cmd = format!("PWD\r\n");
        try!(self.write_str(&pwd_cmd));

        self.read_response(status::PATH_CREATED)
            .and_then(|Line(_, line)| {
                match (line.find('"'), line.rfind('"')) {
                    (Some(begin), Some(end)) if begin < end => {
                        Ok(line[begin+1 .. end].to_string())
                    }
                    _ => {
                        let cause = format!("Invalid PWD Response: {}", line);
                        Err(FtpError::InvalidResponse(cause))
                    }
                }
            })
    }

    /// NOOP command
    pub fn noop(&mut self) -> Result<()> {
        let noop_cmd = format!("NOOP\r\n");
        try!(self.write_str(&noop_cmd));

        self.read_response(status::COMMAND_OK).map(|_| ())
    }

    /// Creates a new directory
    pub fn mkdri(&mut self, path: &str) -> Result<()> {
        let mkdir_cmd = format!("MKD {}\r\n", path);
        try!(self.write_str(&mkdir_cmd));

        self.read_response(status::PATH_CREATED).map(|_| ())
    }

    /// Switches to Passive mode
    pub fn pasv(&mut self) -> Result<SocketAddr> {
        try!(self.write_str("PASV\r\n"));
        let Line(_, line) = try!(self.read_response(status::PASSIVE_MODE));
        PORT_RE.captures(&line)
            .ok_or(FtpError::InvalidResponse(format!("Invalid PASV Response: {}", line)))
            .and_then(|caps| {
                let (oct1, oct2, oct3, oct4) = (
                    caps[1].parse::<u8>().unwrap(),
                    caps[2].parse::<u8>().unwrap(),
                    caps[3].parse::<u8>().unwrap(),
                    caps[4].parse::<u8>().unwrap(),
                );
                let (msb, lsb) = (
                    caps[5].parse::<u8>().unwrap(),
                    caps[6].parse::<u8>().unwrap(),
                );
                let port = ((msb as u16) << 8) + lsb as u16;
                let addr = format!("{}.{}.{}.{}:{}", oct1, oct2, oct3, oct4, port);

                if cfg!(feature = "debug_print") {
                    try!(stdio::stderr().write_fmt(format_args!("PASV Addr: {}\n", addr)));
                }

                SocketAddr::from_str(&addr).map_err(|parse_err| {
                    FtpError::InvalidAddress(parse_err)
                })
            })
    }

    /// Quits the current FTP session
    pub fn quit(&mut self) -> Result<()> {
        let quit_cmd = format!("QUIT\r\n");
        try!(self.write_str(&quit_cmd));

        self.read_response(status::CLOSING).map(|_| ())
    }

    /// Renames the file from from_name to to_name
    pub fn rename(&mut self, from_name: &str, to_name: &str) -> Result<()> {
        let rnfr_cmd = format!("RNFR {}\r\n", from_name);
        try!(self.write_str(&rnfr_cmd));
        self.read_response(status::REQUEST_FILE_PENDING)
            .and_then(|_| {
                let rnto_cmd = format!("RNTO {}\r\n", to_name);
                try!(self.write_str(&rnto_cmd));
                self.read_response(status::REQUESTED_FILE_ACTION_OK).map(|_| ())
            })
    }

    /// Removes a directory
    pub fn rmdir(&mut self, path: &str) -> Result<()> {
        let rmdir_cmd = format!("RMD {}\r\n", path);
        try!(self.write_str(&rmdir_cmd));

        self.read_response(status::REQUESTED_FILE_ACTION_OK).map(|_| ())
    }

    /// Removes a file
    pub fn rm(&mut self, filename: &str) -> Result<()> {
        let rm_cmd = format!("DELE {}\r\n", filename);
        try!(self.write_str(&rm_cmd));

        self.read_response(status::REQUESTED_FILE_ACTION_OK).map(|_| ())
    }

    /// Gets the size of file in bytes, if file doesn't exists, return None
    pub fn size(&mut self, pathname: &str) -> Result<Option<usize>> {
        let size_cmd = format!("SIZE {}\r\n", pathname);
        try!(self.write_str(&size_cmd));

        let Line(_, line) = try!(self.read_response_in(&[status::FILE, status::FILE_UNAVAILABLE]));

        match SIZE_RE.captures(&line) {
            Some(caps) => Ok(Some(caps[1].parse::<usize>().unwrap())),
            None => Ok(None),
        }
    }
}
