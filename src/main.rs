extern crate rand;
extern crate ureq;

use ::std::net::{SocketAddr, IpAddr, Ipv4Addr};
use ::std::process::exit;
use rand::{thread_rng, Rng};
use std::net::{TcpListener, TcpStream};
use std::convert::TryInto;
use std::io::Write;
use std::thread::{spawn, sleep};
use std::sync::Mutex;

static PORT: u16 = 3333u16;
static SLEEP: u64 = 360u64;

pub fn listen() -> TcpListener {
    let ip: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let port: u16 = PORT;
    println!("Listening on port {}", port);

    let addr: SocketAddr = SocketAddr::new(ip, port);
    TcpListener::bind(&addr)
    .expect("unable to bind TCP listener")
}

pub fn check_key(value: &String) -> bool {
    let url: String =
        format!("https://www.googleapis.com/youtube/v3/channels?part=id&id=UC-lHJZR3Gqxm24_Vd_AJ5Yw&key={}", value);
    let path: &str = url.as_str();

    let good: bool = ureq::head(path).call().ok();
    println!("{} {}", value, good);

    good
}

pub fn key_init() -> Vec<String> {
    let keys: Vec<String> = std::env::args().skip(1).collect();
    if keys.is_empty() {
        eprintln!("No keys");
        exit(1);
    } else {
        println!("Got {} keys: {:?}", keys.len(), keys);
        keys
    }
}

pub fn key_status(keys: &Vec<String>) -> Mutex<Vec<bool>> {
    let mut t: Vec<bool> = Vec::new();

    for i in 0..keys.len() {
        let value: &String = &keys[i];
        let value = check_key(value);

        t.push(value);
    }

    Mutex::new(t)
}

fn main() {
    println!("Starting key service");

    let listener: TcpListener = listen();
    let keys: Vec<String> = key_init();
    let len: usize = keys.len();
    let conds: Mutex<Vec<bool>> = key_status(&keys);

    spawn(move || {
        println!("Starting key audit service");
        let secs: u64 = SLEEP;
        let dur = std::time::Duration::from_secs(secs);

        loop {
            for i in 0..len {
                sleep(dur);
                let value: &String = &keys[i];
                let cond: bool = check_key(value);

                {
                    conds.lock().unwrap()[i] = cond;
                }
            }
        }
    });

    for stream in listener.incoming() {
        if stream.is_err() {
            eprintln!("Connection is bad: {:?}", stream);
            exit(3);
        }

        let mut stream: TcpStream = stream.unwrap();
        println!("Got request");
        let n: u32 = thread_rng().gen_range(0, &len).try_into().unwrap();

        let buf: [u8; 1] = [8u8];
        stream.write(&buf)
            .expect("Could not write to socket");
    }
}