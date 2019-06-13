extern crate rand;
extern crate ureq;

use ::std::net::{SocketAddr, IpAddr, Ipv4Addr};
use ::std::process::exit;
use rand::{thread_rng, Rng};
use std::net::{TcpListener, TcpStream};
use std::io::Write;
use std::thread::{spawn, sleep};
use std::time::Duration;
use std::sync::{Arc, Mutex};

static PORT: u16 = 3333u16;
static SLEEP: u64 = 5u64;

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

pub fn key_status(keys: &Vec<String>) -> Arc<Mutex<Vec<bool>>> {
    let mut data: Vec<bool> = Vec::new();

    for i in 0..keys.len() {
        let value: &String = &keys[i];
        let value = check_key(value);

        data.push(value);
    }

    Arc::new(Mutex::new(data))
}

pub fn get_random_key<'a>(keys: &'a Vec<String>, conds: &'a Vec<bool>) -> &'a [u8] {
    let mut good_keys: Vec<&String> = Vec::new();

    for i in 0..keys.len() {
        let value: &String = &keys[i];
        let cond: &bool  = &conds[i];

        if *cond {
            good_keys.push(value);
        }
    }

    let low: usize = 0;
    let high: usize = keys.len();
    let n: usize = thread_rng().gen_range(low, high);

    let key: &String = &keys[n];
    println!("GET {}", key);

    key.as_bytes()
}

fn main() {
    println!("Starting key service");

    let listener: TcpListener = listen();
    let keys1: Arc<Vec<String>> = Arc::new(key_init());
    let keys2: Arc<Vec<String>> = keys1.clone();

    let conds1: Arc<Mutex<Vec<bool>>> = key_status(&keys1);
    let conds2: Arc<Mutex<Vec<bool>>> = Arc::clone(&conds1);

    spawn( move || {
        println!("Starting key audit service");
        let secs: u64 = SLEEP;
        let dur: Duration = std::time::Duration::from_secs(secs);

        loop {
            for i in 0..keys2.len() {
                sleep(dur);
                let value: &String = &keys2[i];
                let cond: bool = check_key(value);

                {
                    conds2.lock().unwrap()[i] = cond;
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
        let conds: Vec<bool> = conds1.lock().unwrap().clone();

        let buf: &[u8] = get_random_key(&keys1, &conds);
        stream.write(&buf)
            .expect("Could not write to socket");
    }
}