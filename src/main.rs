extern crate time;
extern crate rand;
use std::net::{ TcpStream, TcpListener };
use std::io::{ Read, Write };
use rand::Rng;

fn serialize_u32(n: u32) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(4);
    buf.push(((n & 0xff000000) >> 24) as u8);
    buf.push(((n & 0x00ff0000) >> 16) as u8);
    buf.push(((n & 0x0000ff00) >> 8) as u8);
    buf.push(((n & 0x000000ff) >> 0) as u8);
    buf
}

fn deserialize_u32(buf: &[u8]) -> u32 {
    assert_eq!(buf.len(), 4);
    let mut n = 0;
    n += (buf[0] as u32) << 24;
    n += (buf[1] as u32) << 16;
    n += (buf[2] as u32) << 8;
    n += (buf[3] as u32) << 0;
    n
}

fn fill_buf_with_random_data(buf: &mut [u8]) {
    let mut rng = rand::thread_rng();
    for i in 0..buf.len() {
        buf[i] = rng.gen();
    }
}

fn client_main(addr: &str) {
    let mut stream = TcpStream::connect(&addr).unwrap();
    let mut buf = vec![0;1024*128];
    let mut total_bytes = 0;
    let download_start = time::now();
    total_bytes = stream.read(&mut buf).unwrap() - 4;
    let data_size = deserialize_u32(&buf[..4]) as usize;
    println!("data_size: {}", data_size);
    while total_bytes < data_size {
        total_bytes += stream.read(&mut buf).unwrap();
    }
    let download_dur = time::now() - download_start;
    let download_speed = (total_bytes as f64) / 1024. / (download_dur.num_milliseconds() as f64) * 1000.;
    println!("download {} bytes, {} second, {} KiB/s", total_bytes, download_dur.num_seconds(), download_speed);

    let mut upload_buf: Vec<u8> = Vec::with_capacity(total_bytes);
    upload_buf.resize(total_bytes, 0);
    fill_buf_with_random_data(&mut upload_buf);
    let mut n = 0;
    while n < upload_buf.len() {
        println!("{}", n);
        n += stream.write(&upload_buf[n..]).unwrap();
        println!("{}", n);
    }
    assert_eq!(n, total_bytes);
    let mut upload_dur = 0usize;
    {
        assert_eq!(stream.read(&mut buf).unwrap(), 4);
        upload_dur = deserialize_u32(&buf[..4]) as usize;
    }
    let upload_speed = (total_bytes as f64) / 1024. / (upload_dur as f64) * 1000.;
    println!("upload {} bytes, {} second, {} KiB/s", total_bytes, upload_dur / 1000, upload_speed);
}

fn server_main(port: u16, data_size: usize) {
    let listener = TcpListener::bind(("0.0.0.0", port)).unwrap();
    let mut buf: Vec<u8> = Vec::with_capacity(data_size);
    buf.resize(data_size, 0);
    fill_buf_with_random_data(&mut buf);
    for iter in listener.incoming() {
        println!("request comming");
        let mut server = iter.unwrap();
        let mut n = 0;
        {
            let data_size_buf = serialize_u32(data_size as u32);
            assert_eq!(server.write(&data_size_buf).unwrap(), 4);
        }
        println!("write start");
        while n < data_size {
            n += server.write(&buf[n..]).unwrap();
        }
        println!("read start");
        let upload_start = time::now();
        n = 0;
        while n < data_size {
            n += server.read(&mut buf).unwrap();
        }
        let upload_dur = time::now() - upload_start;
        {
            let upload_dur_buf = serialize_u32(upload_dur.num_milliseconds() as u32);
            assert_eq!(server.write(&upload_dur_buf).unwrap(), 4);
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
