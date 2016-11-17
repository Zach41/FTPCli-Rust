use std::io::{Read, Write};
use std::net::{TcpStream, SocketAddr};
use regex::Regex;
use super::status;
use super::types::{FileType, FtpError, Line, Result};

lazy_static! {
    static ref PORT_RE: Regex = Regex::new(r"\((\d+),(\d+),(\d+),(\d+),(\d+),(\d+)\)").unwrap();
    static ref SIZE_RE: Regex = Regex::new(r"\s+(\d+)\s*$").unwrap();
}


