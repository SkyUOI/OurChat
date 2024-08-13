mod test_lib;

#[test]
#[serial_test::serial]
fn test_login_register() {
    // 注册和登录会被自动运行
    test_lib::init_server();
}

cleanup!();
