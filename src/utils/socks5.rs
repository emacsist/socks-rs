pub const VER:u8 = 0x05;
pub const UP_VER:u8 = 0x01;
pub const APP_FILE: &str = "app.json";
pub const USER_PASS_METHOD:u8 = 0x02;
pub const USER_PASS_OK:u8 = 0x00;

pub const REQUEST_CMD_CONNECT:u8 = 0x01;
pub const REQUEST_CMD_BIND:u8 = 0x02;
pub const REQUEST_CMD_UDP:u8 = 0x03;

pub const REP_OK: u8 = 0x00; //表示成功
pub const REP_FAIL: u8 = 0x01; //普通SOCKS服务器连接失败
pub const REP_REFUSE: u8 = 0x02; //现有规则不允许连接
pub const REP_NETWORK_UNREACHABLE: u8 = 0x03; //网络不可达
pub const REP_HOST_UNREACHABLE: u8 = 0x04; //主机不可达
pub const REP_CONNECT_REFUSE: u8 = 0x05; //连接被拒
pub const REP_TTL_TIMEOUT: u8 = 0x06; // TTL超时
pub const REP_UNSUPPORT_CMD: u8 = 0x07; //不支持的命令
pub const REP_UNSUPPORT_ADDR: u8 = 0x08; //不支持的地址类型
pub const REP_UNKNOWN: u8 = 0x09; // 0x09 - 0xFF未定义

pub const ATYPE_IP4: u8 = 0x01;
pub const ATYPE_DOMAINNAME: u8 = 0x03;
pub const ATYPE_IP6: u8 = 0x04;
