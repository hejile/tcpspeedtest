extern crate time;
use std::net::{ TcpStream, TcpListener, SocketAddr, ToSocketAddrs };
use std::io::{ Read, Write };

fn client_main(addr: &str) {
    let mut stream = TcpStream::connect(&addr).unwrap();
    let mut buf = vec![0;1024*128];
    let mut total_bytes = 0;
    let start = time::now();
    loop {
        let n = stream.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        total_bytes += n;
    }
    let dur = time::now() - start;
    let speed = (total_bytes as f64) / 1024. / (dur.num_milliseconds() as f64);
    println!("{} bytes, {} second, {} KiB/s", total_bytes, dur.num_seconds(), speed);
}

fn server_main(port: u16) {
    let listener = TcpListener::bind(("0.0.0.0", port)).unwrap();
    let buf = vec![0; 1024*1024*10];
    for iter in listener.incoming() {
        println!("request comming");
        let mut server = iter.unwrap();
        let mut n = 0;
        while n < buf.len() {
            n += server.write(&buf).unwrap();
        }
        println!("request end");
    }
}


fn help() {
    println!("
-c host:port
-s port");
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() != 3 {
        help();
        return;
    }
    if args[1] == "-c" {
        let addr = &args[2];
        client_main(addr);
    } else if args[1] == "-s" {
        let port: u16 = args[2].parse().unwrap();
        server_main(port);
    } else {
        help();
        return;
    }
}
