mod test_lib;

async fn test_login_register() {
    // 注册和登录会被自动运行
    test_lib::get_connection().await;
}

register_test!(test_login_register);
