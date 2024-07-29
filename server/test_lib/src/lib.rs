use std::{net::TcpStream, sync::OnceLock, thread};

fn set_up_server() {
    let mut cmd = assert_cmd::Command::cargo_bin("server").unwrap();
    let _ = cmd.arg("--cfg").arg("../config/ourchat.toml").assert();
}

fn init_server() {
    static TMP: OnceLock<()> = OnceLock::new();
    TMP.get_or_init(|| {
        thread::spawn(|| set_up_server());
    });
}

pub fn connent_to_server() -> TcpStream {
    init_server();
    let socket = TcpStream::connect("127.0.0.1:7777").unwrap();
    socket
}
