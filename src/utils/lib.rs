pub mod socks5;

use log::{error, info, debug};
use std::net::{AddrParseError, IpAddr, TcpStream};
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Read;

const APP_FILE: &str = "app.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct App {
    pub server: String,
    pub server_port: u16,
    pub local_address: String,
    pub local_port: u16,
    pub username: String,
    pub password: String,
    pub timeout: u64,
    pub method: String
}

impl App {
    pub fn read_app() -> Self {
        let mut file = File::open(APP_FILE).unwrap();
        let mut data = String::with_capacity(128);
        file.read_to_string(&mut data).unwrap();
        serde_json::from_str(data.as_str()).unwrap()
    }

    pub fn full_server(&self) -> String {
        format!("{}:{}", self.server, self.server_port)
    }

    pub fn server_addr(&self) -> Option<IpAddr> {
        let ipAddr: Result<IpAddr, AddrParseError> = self.server.parse();
        match ipAddr {
            Ok(ip) => {
                Some(ip)
            },
            Err(err) => {
                error!("invalid server addr! {:?}", err);
                None
            }
        }
    }

    pub fn full_local(&self) -> String {
        format!("{}:{}", self.local_address, self.local_port)
    }
}

pub fn read_stream(stream: &mut TcpStream) -> (Option<[u8;10240]>, usize) {
    let mut buffer = [0; 10240];
    match stream.read(&mut buffer) {
        Ok(n) => {
            if n == 0 {
                debug!("read_stream data empty. so return! {:?}", stream);
                return (None, 0);
            }
            debug!("read {} bytes, {:?} => {:?}", n, stream, buffer.as_ref());
            return (Some(buffer), n);
        },
        Err(err) => {
            error!("read_stream error! {:?}", stream);
            return (None, 0);
        }
    };

}