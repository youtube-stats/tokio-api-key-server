extern crate rand;
extern crate tokio;
extern crate ureq;

use ::std::net::{SocketAddr, IpAddr, Ipv4Addr};
use ::std::process::exit;
use rand::{thread_rng, Rng};
use crate::tokio::executor::spawn;
use crate::tokio::io::write_all;
use crate::tokio::net::{TcpListener,TcpStream};
use crate::tokio::prelude::{Future,Stream};
use crate::tokio::run;

static PORT: u16 = 3333u16;

fn main() {
    let listener: TcpListener = {
        let ip: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        let port: u16 = PORT;

        let addr: SocketAddr = SocketAddr::new(ip, port);
        TcpListener::bind(&addr)
            .expect("unable to bind TCP listener")
    };
    let keys: Vec<String> = {
        let keys: Vec<String> = std::env::args().skip(1).collect();
        if keys.is_empty() {
            eprintln!("No keys passed");
            exit(1);
        }

        println!("Got {} keys: {:?}", keys.len(), keys);
        let mut good_keys: Vec<String> = Vec::new();
        for value in keys {
            let url: String =
                format!("https://www.googleapis.com/youtube/v3/channels?part=id&id=UC-lHJZR3Gqxm24_Vd_AJ5Yw&key={}", value);
            let path: &str = url.as_str();

            let resp = ureq::head(path).call();

            if resp.ok() {
                println!("{} is good", value);
                good_keys.push(value);
            }
        }

        println!("Keeping {} keys", good_keys.len());
        good_keys
    };

    let future = listener.incoming()
        .map_err(|e| eprintln!("accept failed = {:?}", e))
        .for_each(move |a: TcpStream| {
            let n: usize = thread_rng().gen_range(0, keys.len());
            let buf: String = keys[n].clone();

            let f = write_all(a, buf).then(move|result| {
                if result.is_ok() {
                    Ok(())
                } else {
                    let err = result.is_err();
                    eprintln!("failed to write to stream: {:?}", err);

                    Err(())
                }
            });

            spawn(f)
        });

    run(future);
}