use std::net::{SocketAddrV4, Ipv4Addr, TcpListener, TcpStream, SocketAddr, IpAddr, Ipv6Addr};
use std::io::{Read, Error, Write};
use once_cell::sync::Lazy;
use utils::App;
use log::{error, info, debug};
use utils::socks5;
use std::time::Duration;
use std::convert::TryInto;


static APP: Lazy<App> = Lazy::new(|| {
    App::read_app()
});

fn main() {
    std::env::set_var("RUST_LOG", "TRACE");
    env_logger::init();
    bind();

}

fn bind() {
    let bind_result = TcpListener::bind(APP.full_server());
    match bind_result {
        Ok(listener) => {
            debug!("ss server started with {}", APP.full_server());
            for stream in listener.incoming() {
                match stream {
                    Ok(mut ss_local_stream) => {
                        debug!("ss local connected! {:?}", ss_local_stream);
                        handle_connection(&mut ss_local_stream);
                    },
                    Err(err) => {
                        error!("stream error! {}", err);
                    }
                }
            }
        },
        Err(err) => {
            error!("listen {} error! {:?}", APP.full_server(), err);
        }
    };
}

fn handle_connection(ss_local_stream: &mut TcpStream) {
    let (mut buffer, n) = utils::read_stream(ss_local_stream);
    if let Some(mut buf) = buffer {
        if !decode_sslocal_handshake(&mut buf[0..n]) {
            return;
        }

        //返回给 ss local， 确认是 user/pass 认证方式
        let mut data = [socks5::VER, socks5::USER_PASS_METHOD];
        ss_local_stream.write_all(&mut data);
        debug!("select u/p method ok...");
        //检验 user/pass
        if !decode_user_passwd(ss_local_stream) {
            return;
        }

        // 连接客户端请求的真正远程地址
        decode_sslocal_request(ss_local_stream);
    }  else {
        error!("ss local no data. so return. {:?}", ss_local_stream);
    }

}

/// 请求的格式
///         +----+-----+-------+------+----------+----------+
///         |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
///         +----+-----+-------+------+----------+----------+
///         | 1  |  1  | X'00' |  1   | Variable |    2     |
///         +----+-----+-------+------+----------+----------+
///
/// 响应的格式
///         +----+-----+-------+------+----------+----------+
///         |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
///         +----+-----+-------+------+----------+----------+
///         | 1  |  1  | X'00' |  1   | Variable |    2     |
///         +----+-----+-------+------+----------+----------
///
fn decode_sslocal_request(ss_local_stream: &mut TcpStream) {
    let (mut buffer, _) = utils::read_stream(ss_local_stream);
    if let Some(mut buf) = buffer {
        let ver = buf[0];
        if ver != socks5::VER {
            ss_local_stream.write_all(&[socks5::VER, 0xff]);
            return;
        }
        let cmd = buf[1];
        // 只支持 CONNECT 命令
        if cmd != socks5::REQUEST_CMD_CONNECT {
            error!("invalid cmd: {}", cmd);
            ss_local_stream.write_all(&[socks5::VER, 0xff]);
            return;
        }
        //let rsv = buf[2];
        let atype = buf[3];
        let remote_stream = match atype {
            socks5::ATYPE_IP4 => {
                let dst_addr:[u8; 4] = (&buf[4..8]).try_into().unwrap();
                let port = u16::from_be_bytes([buf[8], buf[9]]);
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from(dst_addr)), port);
                let remote_stream = TcpStream::connect_timeout(&addr, Duration::from_secs(APP.timeout));
                remote_stream
            },
            socks5::ATYPE_IP6 => {
                let dst_addr:[u8; 16] = (&buf[4..20]).try_into().unwrap();
                let port = u16::from_be_bytes([buf[8], buf[9]]);
                let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::from(dst_addr)), port);
                let remote_stream = TcpStream::connect_timeout(&addr, Duration::from_secs(APP.timeout));
                remote_stream
            },
            _ => {
                error!("unkonw atype: {}", atype);
                return;
            }
        };

        match remote_stream {
            Ok(mut remote_tcp_stream) => {
                debug!("connected to dst: {:?}", remote_tcp_stream);
                response_requst_ok(ss_local_stream, &mut remote_tcp_stream);
                loop {
                    if let (Some(mut data), n) = utils::read_stream(ss_local_stream) {
                        remote_tcp_stream.write_all(&mut data[0..n]);
                    } else {
                        info!("ss local no data to read. so return. {:?}", ss_local_stream);
                        break;
                    }
                    if let (Some(mut data), n) = utils::read_stream(&mut remote_tcp_stream) {
                        ss_local_stream.write_all(&mut data[0..n]);
                    } else {
                        info!("remote server no data return. so exit. {:?}", remote_tcp_stream);
                        break;
                    }
                }
                debug!("server copy remote -> ss local stream ok...");
            },
            Err(err) => {
                error!("connect to remote addr error! => {:?}", err);
                return;
            }
        }
    } else {
        error!("ss local no request...!. so return. {:?}", ss_local_stream);
    }
}

/// 响应的格式
///         +----+-----+-------+------+----------+----------+
///         |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
///         +----+-----+-------+------+----------+----------+
///         | 1  |  1  | X'00' |  1   | Variable |    2     |
///         +----+-----+-------+------+----------+----------
///
fn response_requst_ok(ss_local_stream: &mut TcpStream, remote_stream: &mut TcpStream) {
    let remote_addr = remote_stream.peer_addr().unwrap();
    debug!("response request: {:?}", remote_addr);
    match remote_addr.ip() {
        IpAddr::V4(ip4) => {
            let ip_bytes = ip4.octets();
            let mut data= [&[0x05u8, socks5::REP_OK, 0x00, socks5::ATYPE_IP4], &ip_bytes[..], &remote_addr.port().to_be_bytes()].concat();
            ss_local_stream.write_all(&data);
        },
        IpAddr::V6(ip6) => {
            let ip_bytes = ip6.octets();
            let mut data= [&[0x05u8, socks5::REP_OK, 0x00, socks5::ATYPE_IP6], &ip_bytes[..], &remote_addr.port().to_be_bytes()].concat();
            ss_local_stream.write_all(&data);
        }
    }
}

///
/// req:
///            +----+------+----------+------+----------+
///            |VER | ULEN |  UNAME   | PLEN |  PASSWD  |
///            +----+------+----------+------+----------+
///            | 1  |  1   | 1 to 255 |  1   | 1 to 255 |
///            +----+------+----------+------+----------+
///
/// resp: status 为 0 表示成功。其他表示失败
///                        +----+--------+
//                         |VER | STATUS |
//                         +----+--------+
//                         | 1  |   1    |
//                         +----+--------+
///
fn decode_user_passwd(ss_local_stream: &mut TcpStream) -> bool {
    let (mut buffer, _) = utils::read_stream(ss_local_stream);
    if let Some(mut buf) = buffer {
        let ver = buf[0];
        //注意， 用户名/密码 认证时，  VER 为1
        if ver != socks5::UP_VER {
            ss_local_stream.write_all(&[socks5::VER, 0xff]);
            return false;
        }
        let ulen = buf[1] as usize;
        if ulen == 0 {
            ss_local_stream.write_all(&[socks5::VER, 0xff]);
            return false;
        }
        let u_offset_end = 2 + ulen as usize;
        let user = String::from_utf8_lossy(&buf[2..u_offset_end]).to_string();

        let plen = buf[u_offset_end] as usize;

        let p_offset_end = u_offset_end + plen + 1;
        let pass = String::from_utf8_lossy(&buf[(u_offset_end + 1)..p_offset_end]).to_string();
        debug!("ss local user: [{}], passoword: [{}] ,server user: [{}], passoword: [{}]", user, pass, APP.username, APP.password);
        if user == APP.username && pass == APP.password {
            debug!("auth u/p ok...");
            ss_local_stream.write_all(&[socks5::VER, socks5::USER_PASS_OK]);
        } else {
            //如果不对， 则返回 0xff，让客户端断开
            error!("auth u/p wrong!...");
            ss_local_stream.write_all(&[socks5::VER, 0xff]);
        }

    }  else {
        error!("ss local no data. so return. {:?}", ss_local_stream);
    }
    true
}

/// client 发送的请求格式为
///                   +----+----------+----------+
//                    |VER | NMETHODS | METHODS  |
//                    +----+----------+----------+
//                    | 1  |    1     | 1 to 255 |
//                    +----+----------+----------+
fn decode_sslocal_handshake(buffer: &mut [u8]) -> bool {
    let ver = buffer[0];
    let nmethods = buffer[1];

    let offset_end = 2 + nmethods as usize;
    let method = &buffer[2..offset_end];

    if ver != socks5::VER {
        error!("invalid ss version! {}", ver);
        return false;
    }

    if nmethods == 0 {
        error!("invalid ss nmethods! {}", nmethods);
        return false;
    }

    if !method.contains(&socks5::USER_PASS_METHOD) {
        error!("not user/password method! {:?}", method);
        return false;
    }
    info!("ss client auth ok...");
    return true;
}