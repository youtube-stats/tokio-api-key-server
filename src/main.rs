extern crate tokio;

use crate::tokio::executor::spawn;
use crate::tokio::io::write_all;
use crate::tokio::net::{TcpListener,TcpStream};
use crate::tokio::prelude::{Future,Stream};
use crate::tokio::run;
use ::std::net::{SocketAddr, IpAddr, Ipv4Addr};
use ::std::process::exit;

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
        println!("Got keys {:?}", keys);

        if keys.is_empty() {
            eprintln!("No keys passed");
            exit(1);
        }

        keys
    };

    let future = listener.incoming()
        .map_err(|e| eprintln!("accept failed = {:?}", e))
        .for_each(|stream: TcpStream| {
            let f = write_all(stream, "hello world\n").then(|result| {
                println!("wrote to stream; success={:?}", result.is_ok());
                Ok(())
            });

            spawn(f)
        });

    run(future);
}