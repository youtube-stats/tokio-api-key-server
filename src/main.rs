extern crate tokio;

use crate::tokio::executor::spawn;
use crate::tokio::io::AsyncRead;
use crate::tokio::io::copy;
use crate::tokio::net::TcpListener;
use crate::tokio::prelude::Future;
use crate::tokio::prelude::Stream;
use crate::tokio::run;

fn main() {
    // Bind the server's socket.
    let addr = "127.0.0.1:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr)
        .expect("unable to bind TCP listener");

    // Pull out a stream of sockets for incoming connections
    let future = listener.incoming()
        .map_err(|e| eprintln!("accept failed = {:?}", e))
        .for_each(|sock| {
            // Split up the reading and writing parts of the
            // socket.
            let (reader, writer) = sock.split();

            // A future that echos the data and returns how
            // many bytes were copied...
            let bytes_copied = copy(reader, writer);

            // ... after which we'll print what happened.
            let f = bytes_copied.map(|amt| {
                println!("wrote {:?} bytes", amt)
            }).map_err(|err| {
                eprintln!("IO error {:?}", err)
            });

            // Spawn the future as a concurrent task.
            spawn(f)
        });

    // Start the Tokio runtime
    run(future);
}