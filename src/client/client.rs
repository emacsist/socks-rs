use std::net::{SocketAddrV4, Ipv4Addr, TcpListener, TcpStream};
use std::io::{self, Read, Error, Write, BufReader, BufWriter};
use once_cell::sync::Lazy;
use utils::App;
use log::{error, info, debug};
use std::time::Duration;
use utils::socks5;

static APP: Lazy<App> = Lazy::new(|| {
    App::read_app()
});

fn main() {
    std::env::set_var("RUST_LOG", "TRACE");
    env_logger::init();
    bind();

}

fn bind() {
    let bind_result = TcpListener::bind(APP.full_local());
    match bind_result {
        Ok(listener) => {
            info!("ss local started with {}", APP.full_local());
            for stream in listener.incoming() {
                match stream {
                    Ok(mut ss_client_stream) => {
                        info!("ss client connected! {:?}", ss_client_stream);
                        handle_client_connection(ss_client_stream);
                    },
                    Err(err) => {
                        error!("stream error! {}", err);
                    }
                }
            }
        },
        Err(err) => {
            error!("ss local listen {} error! {:?}", APP.full_server(), err);
        }
    };
}


/// 目前只是单纯转发
fn handle_client_connection(mut ss_client_stream: TcpStream) {
    let server_addr = APP.full_server().parse();
    let ss_server_stream = TcpStream::connect_timeout(&server_addr.unwrap(), Duration::from_secs(APP.timeout));
    match ss_server_stream {
        Ok(mut server_stream) => {
            loop {
                if let (Some(mut data), n) = utils::read_stream(&mut ss_client_stream) {
                    debug!("eco to server stream...{:?}", data.as_ref());
                    server_stream.write_all(&mut data[0..n]);
                } else {
                    info!("no data to read. so return. {:?}", ss_client_stream);
                    break;
                }

                if let (Some(mut data), n) = utils::read_stream(&mut server_stream) {
                    debug!("eco to ss client stream...{:?}", data.as_ref());
                    ss_client_stream.write_all(&mut data[0..n]);
                } else {
                    info!("server no data return. so exit. {:?}", ss_client_stream);
                    break;
                }

            }
            debug!("copy stream ok...");
        },
        Err(err) => {
            error!("connect to ss server error! {:?}", err);
        }
    }
}
