extern crate time;
use std::net::{ TcpStream, TcpListener, SocketAddr, ToSocketAddrs };
use std::io::{ Read, Write };

fn client_main(addr: &str) {
    let mut stream = TcpStream::connect(&addr).unwrap();
    let mut buf = vec![0;1024*128];
    let mut total_bytes = 0;
    let download_start = time::now();
    loop {
        let n = stream.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        total_bytes += n;
    }
    let download_dur = time::now() - download_start;
    let download_speed = (total_bytes as f64) / 1024. / (download_dur.num_milliseconds() as f64) * 1000.;
    println!("download {} bytes, {} second, {} KiB/s", total_bytes, download_dur.num_seconds(), download_speed);

    let mut upload_buf: Vec<u8> = Vec::with_capacity(total_bytes);
    upload_buf.resize(total_bytes, 0);
    let upload_start = time::now();
    let mut n = 0;
    while n < buf.len() {
        n += stream.write(&upload_buf[n..]).unwrap();
    }
    assert_eq!(n, total_bytes);
    let upload_dur = time::now() - upload_start;
    let upload_speed = (total_bytes as f64) / 1024. / (upload_dur.num_milliseconds() as f64) * 1000.;
    println!("upload {} bytes, {} second, {} KiB/s", total_bytes, upload_dur.num_seconds(), upload_speed);
}

fn server_main(port: u16, data_size: usize) {
    let listener = TcpListener::bind(("0.0.0.0", port)).unwrap();
    let mut buf: Vec<u8> = Vec::with_capacity(data_size);
    buf.resize(data_size, 0);
    for iter in listener.incoming() {
        println!("request comming");
        let mut server = iter.unwrap();
        let mut n = 0;
        println!("write start");
        while n < buf.len() {
            n += server.write(&buf[n..]).unwrap();
        }
        server.shutdown(std::net::Shutdown::Write).unwrap();
        println!("read start");
        loop {
            let n = server.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }   
        }
        println!("request end");
    }
}


fn help() {
    println!("
-c host:port
-s port (datasize)?");
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 3 {
        help();
        return;
    }
    if args[1] == "-c" {
        let addr = &args[2];
        client_main(addr);
    } else if args[1] == "-s" {
        let port: u16 = args[2].parse().unwrap();
        let data_size: usize = if args.len() == 4 { args[3].parse().unwrap() } else { 1024*1024*2 };
        server_main(port, data_size);
    } else {
        help();
        return;
    }
}
