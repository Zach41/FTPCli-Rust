# FTP client implemented in Rust

## Abstract

A FTP client implemented in Rust.

the main interface used in the client is learning from [rust-ftp](https://github.com/mattnenterprise/rust-ftp). 

Right now, this client implemented following FTP commands:

- `login`: login a user with usename and password.
- `exit`: exit the FTP session.
- `ls`: list contents of the remote directory.
- `pwd`: print working directory remote machine.
- `get`: retrive file.
- `put`: send one file.
- `cd`: change remote working directory.
- `help`: print local help information, `help cmd` will print specific command help information.
- `cdup`: change remote working directory to parent directory.
- `mkdir`: make directory on the remote machine.
- `rmdir`: remove directory on the remote machine.
- `delete`: delete remote file.
- `size`: show the size of remote file.
- `nlist`: nlist contents of remote file.
- `modtime`: show last modification time of remote file.

## Usage

You must have Rust and Cargo installed on your machine. 

```shell
git clone git@github.com:Zach41/FTPCli-Rust.git

cargo install	
```

Then, you're ready to go.

```shell
$ zftp 127.0.0.1
220 (vsFTPd 3.0.3)
Connected to 127.0.0.1
ftp> 
```