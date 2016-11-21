extern crate FTPCLI;
extern crate chrono;
extern crate rpassword;
#[macro_use] extern crate lazy_static;

use std::io as stdio;
use std::fs::File;
use std::io::{Read, Write, BufReader};
use std::process::exit;
use std::env;
use std::iter::Iterator;
use std::collections::HashMap;
use chrono::{Timelike, Datelike};
use rpassword::prompt_password_stdout;

use FTPCLI::{FtpStream, FtpError, status};

lazy_static! {
    static ref CMD_SET: Vec<&'static str> = {
        let mut cmds: Vec<&str> = Vec::new();
        cmds.push("login");
        cmds.push("exit");
        cmds.push("ls");
        cmds.push("pwd");
        cmds.push("get");
        cmds.push("get");
        cmds.push("put");
        cmds.push("cd");
        cmds.push("help");
        cmds.push("cdup");
        cmds.push("mkdir");
        cmds.push("rmdir");
        cmds.push("delete");
        cmds.push("size");
        cmds.push("nlist");
        cmds.push("modtime");
        
        cmds
    };
    
    static ref CMD_INFO: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("login", "user login command");
        map.insert("exit", "terminate ftp session and exit");
        map.insert("ls", "list contents of remote directory");
        map.insert("pwd", "print working directory on remote machine");
        map.insert("get", "retrive file");
        map.insert("put", "send one file");
        map.insert("cd", "change remote working directory");
        map.insert("help", "print local help information");
        map.insert("cdup", "change remote working directory to parent directory");
        map.insert("mkdir", "make directory on the remote machine");
        map.insert("rmdir", "remove directory on the remote machine");
        map.insert("delete", "delete remote file");
        map.insert("size", "show the size of remote file");
        map.insert("nlist", "nlist contents of remote directory");
        map.insert("modtime", "show last modification time of remote file");
        
        map
    };
}


fn cmd_loop(ftp_stream: &mut FtpStream) -> ! {
    'looper: loop {
        print!("ftp> ");
        stdio::stdout().flush().unwrap();
        
        let mut cmd_line = String::new();
        stdio::stdin().read_line(&mut cmd_line).unwrap();

        let cmds: Vec<String> = cmd_line.split(' ').into_iter()
            .map(|s| String::from(s.trim()))
            .filter(|s| s.len() > 0).collect();

        if cfg!(feature = "debug_print") {
            println!("CMD {}", cmds[0]);
        }
        
        match cmds[0].as_ref() {
            "login"=> {
                login(ftp_stream);
            },
            "exit" => {
                println!("Bye");
                break 'looper;
            },
            "ls" => {
                let path = match cmds.capacity() {
                    1 => Some("."),
                    _ => Some(cmds[1].as_ref()),
                };
                ls(ftp_stream, path);
            },
            "pwd" => {
                pwd(ftp_stream);
            },
            "get" => {
                if cmds.capacity() <= 1 {
                    println!("Invalid arguements");
                    continue;
                }
                let (src, desc) = match cmds.capacity() {
                    2 => (cmds[1].clone(), cmds[1].clone()),
                    _ => (cmds[1].clone(), cmds[2].clone()),                  
                };
                get(ftp_stream, &src, &desc);
            },
            "put" => {
                match cmds.capacity() {
                    1 => {
                        println!("Invalid arguements");
                        continue;
                    }
                    2 => {
                        put(ftp_stream, &cmds[1], &cmds[1]);
                    }
                    _ => {
                        put(ftp_stream, &cmds[1], &cmds[2]);
                    }
                }
            },
            "cd" => {
                match cmds.capacity() {
                    1 => {
                        println!("Invalid arguements");
                        continue;
                    }
                    _ => {
                        cd(ftp_stream, &cmds[1]);
                    }
                }
            },
            "help" => {
                match cmds.capacity() {
                    1 => {
                        let mut cnt = 0;
                        for cmd in CMD_SET.iter() {
                            print!("{:8}", cmd);
                            cnt += 1;
                            if cnt % 8 == 0 {
                                print!("\n");
                            }
                        }
                        if cnt % 8 != 0 {
                            print!("\n");
                        }
                    }
                    _ => {
                        let cmd = cmds[1].clone();                        
                        println!("{}\t\t{}", cmd, CMD_INFO.get(&cmd[..]).unwrap());
                    }
                }
            },
            "cdup" => {
                match ftp_stream.cdup() {
                    Ok(()) => (()),
                    Err(_) => println!("cdup command failed"),
                }
            },
            "mkdir" => {
                match cmds.capacity() {
                    1 => println!("Invalid arguements"),
                    _ => {
                        match ftp_stream.mkdir(&cmds[1]) {
                            Ok(()) => (()),
                            Err(_) => println!("mkdir command failed"),
                        }
                    },
                }
            },
            "rmdir" => {
                match cmds.capacity() {
                    1 => println!("Invalid arguements"),
                    _ => {
                        match ftp_stream.rmdir(&cmds[1]) {
                            Ok(()) => (()),
                            Err(_) => println!("rmdir command failed"),
                        }
                    }
                }
            },
            "delete" => {
                match cmds.capacity() {
                    1 => println!("Invalid arguements"),
                    _ => {
                        match ftp_stream.rm(&cmds[1]) {
                            Ok(()) => (()),
                            Err(_) => println!("delete command failed"),
                        }
                    }
                }
            },
            "size" => {
                match cmds.capacity() {
                    1 => println!("Invalid arguements"),
                    _ => size(ftp_stream, &cmds[1]),
                }
            },
            "nlist" => {
                match cmds.capacity() {
                    1 => nlist(ftp_stream, None),
                    _ => nlist(ftp_stream, Some(&cmds[1])),
                }
            },
            "modtime" => {
                match cmds.capacity() {
                    1 => println!("Invalid arguements"),
                    _ => modtime(ftp_stream, &cmds[1]),
                }
            },
            _ => {
                println!("Invalid command or not implemented!");
            }
        }
    }
    exit(-1);
}

fn login(ftp_stream: &mut FtpStream) {
    print!("Name: ");
    stdio::stdout().flush().unwrap();
    let mut name: String = String::new();
    // let mut passwd: String = String::new();

    stdio::stdin().read_line(&mut name).unwrap();
    match ftp_stream.user(&name) {
        Ok(()) => (()),
        Err(_) => { println!("Login failed"); return; }
    };

    let passwd = prompt_password_stdout("Password: ").unwrap();

    match ftp_stream.pass(&passwd) {
        Ok(()) => println!("Login successfully"),
        Err(_) => println!("Login failed"),
    }
}

fn ls(ftp_stream: &mut FtpStream, pathname: Option<&str>) {
    let list: Vec<String> = match ftp_stream.list(pathname) {
        Ok(list) => list,
        Err(_) => {
            println!("no such file or directory");
            return;
        }
    };

    for item in list {
        println!("{}", item);
    }
}

fn pwd(ftp_stream: &mut FtpStream) {
    match ftp_stream.pwd() {
        Ok(pwd) => println!("{}", pwd),
        Err(_) => println!("pwd command error"),
    }
}

fn get(ftp_stream: &mut FtpStream, src: &str, desc: &str) {
    match ftp_stream.retr(src, |stream| {
        let mut file = File::create(desc).unwrap();
        let mut buf = [0; 2048];
        
        loop {
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => file.write_all(&buf[0..n]),
                Err(err) => return Err(FtpError::ConnectionError(err))
            }.unwrap();
        }

        Ok(())
    }) {
        Ok(()) => (()),
        Err(_) => println!("get file {} failed", src),
    }
}

fn cd(ftp_stream: &mut FtpStream, pathname: &str) {
    match ftp_stream.cwd(pathname) {
        Ok(()) => (()),
        Err(_) => println!("change directory failed"),
    }
}

fn put(ftp_stream: &mut FtpStream, src: &str, desc: &str) {
    match File::open(src) {
        Ok(file) => {
            let mut reader = BufReader::new(file);

            match ftp_stream.put(desc, &mut reader) {
                Ok(()) => (()),
                Err(_) => println!("put file failed"),
            }
        }
        Err(err) => println!("open file failed: {}", err.to_string())
    };
}

fn size(ftp_stream: &mut FtpStream, filename: &str) {
    match ftp_stream.size(filename) {
        Ok(opsize) => {
            match opsize {
                Some(size) => println!("{}: {}", filename, size),
                None => println!("no such file or directory"),
            }
        }
        Err(_) => println!("size command failed"),
    }
}

fn nlist(ftp_stream: &mut FtpStream, pathname: Option<&str>) {
    match ftp_stream.nlist(pathname) {
        Ok(files) => {
            let mut cnt = 0;
            for file in files {
                print!("{}\t", file);
                cnt += 1;
                if cnt % 8 == 0 {
                    print!("\n");
                }                
            }
            if cnt % 8 != 0 {
                print!("\n");
            }
        }
        Err(_) => println!("nlist command failed"),
    }
}

fn modtime(ftp_stream: &mut FtpStream, filename: &str) {
    match ftp_stream.mdtm(filename) {
        Ok(optime) => {
            match optime {
                Some(time) => {
                    println!("{} {}/{}/{} {}:{}:{} GMT", filename,
                             time.day(), time.month(), time.year(),
                             time.hour(), time.minute(), time.second());
                },
                None => (()),
            }
        },
        Err(_) => println!("modtime command error"),
    }
}

fn main() {
    let argc = env::args().count();
    
    match argc {
        1 => {
            println!("Usage: ftp <IP> [PORT]");
            exit(-1);
        }
        _ => {
            let mut args = env::args();
            let ip = args.nth(1).unwrap();
            let port = match args.next() {
                Some(port) => port,
                None => "21".to_string(),
            };
            let addr = format!("{}:{}", ip, port);
            match FtpStream::connect(&addr[..]) {
                Ok(mut stream) => {
                    println!("Connected to {}", ip);
                    cmd_loop(&mut stream)
                }
                Err(err) => {
                    println!("Connection Failed: {}", err.to_string());
                    exit(-1);
                }
            };
        }
    }
}
