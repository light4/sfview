#![feature(seek_convenience)]

use anyhow::Result;
use ssh2::Session;
use std::io::{Read, Seek, SeekFrom};
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::path::Path;
use std::{thread, time};

const DURATION: time::Duration = time::Duration::from_secs(1);

fn watch_file<V>(mut file: V) -> Result<()>
where
    V: Read + Seek,
{
    loop {
        let cur = file.stream_position()?;

        let mut buf = vec![];
        match file.read_to_end(&mut buf)? {
            0 => {
                thread::sleep(DURATION);
                file.seek(SeekFrom::Start(cur))?;
            }
            _ => {
                let s = String::from_utf8_lossy(&buf);
                print!("{}", &s);
            }
        };
    }
}

fn exec_cmd(sess: &Session) -> Result<()> {
    let mut channel = sess.channel_session()?;
    channel.exec("ls")?;
    let mut s = String::new();
    channel.read_to_string(&mut s)?;
    println!("{}", s);
    channel.wait_close()?;
    println!("{}", channel.exit_status()?);

    Ok(())
}

fn get_session<A: ToSocketAddrs>(addr: A) -> Result<Session> {
    // Connect to the local SSH server
    let tcp = TcpStream::connect(addr)?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_agent("user")?;
    Ok(sess)
}

fn main() -> Result<()> {
    let addr = "localhost:22";
    let log_path = "/var/test/test.log";

    let sess = get_session(addr)?;

    exec_cmd(&sess)?;

    let sftp = sess.sftp()?;
    let file = sftp.open(Path::new(log_path))?;
    // let file = std::fs::File::open(Path::new("test.log"))?;
    watch_file(file)?;

    Ok(())
}
