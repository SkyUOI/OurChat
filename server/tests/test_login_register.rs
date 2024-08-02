use futures_util::{SinkExt, StreamExt};
use server::{
    connection::client_response::{self, ErrorMsgResponse, LoginResponse},
    consts::MessageType,
};

#[test]
#[serial_test::serial]
fn test_login_register() {
    // 注册和登录会被自动运行
    test_lib::init_server();
}

test_lib::cleanup!();
