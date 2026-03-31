import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/core/database.dart';
import 'package:ourchat/server_setting.dart';
import 'core/account.dart';
import 'core/auth_notifier.dart';

// Auth界面
class Auth extends StatelessWidget {
  const Auth({super.key});
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: DefaultTabController(
          length: 2,
          child: Column(
            children: [
              TabBar(
                tabs: [
                  Tab(text: l10n.login),
                  Tab(text: l10n.register),
                ],
              ),
              const Expanded(
                child: TabBarView(children: [Login(), Register()]),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class Login extends ConsumerStatefulWidget {
  const Login({super.key});

  @override
  ConsumerState<Login> createState() => _LoginState();
}

class _LoginState extends ConsumerState<Login> {
  String account = "", password = "", avatarUrl = "";
  bool savePassword = false, inited = false;
  @override
  Widget build(BuildContext context) {
    var key = GlobalKey<FormState>();
    final config = ref.read(configProvider);
    if (!inited) {
      account = config.recentAccount;
      password = config.recentPassword;
      avatarUrl = config.recentAvatarUrl;
      if (password.isNotEmpty) savePassword = true;
      inited = true;
    }
    return SafeArea(
      child: Form(
        key: key,
        child: Row(
          children: [
            Flexible(flex: 1, child: Container()),
            Flexible(
              flex: 3,
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Padding(
                    padding: EdgeInsets.all(10.0),
                    child: SizedBox(
                      height: 100.0,
                      width: 100.0,
                      child: (avatarUrl.isEmpty
                          ? Image.asset("assets/images/logo.png")
                          : UserAvatar(
                              imageUrl: avatarUrl,
                              size: AppStyles.largeAvatarSize,
                            )),
                    ),
                  ),
                  TextFormField(
                    // 账号输入框
                    initialValue: account,
                    decoration: InputDecoration(
                      label: Text("${l10n.ocid}/${l10n.email}"),
                    ),
                    onSaved: (newValue) {
                      setState(() {
                        account = newValue!;
                      });
                    },
                  ),
                  TextFormField(
                    // 密码输入框
                    initialValue: password,
                    decoration: InputDecoration(label: Text(l10n.password)),
                    onSaved: (newValue) {
                      setState(() {
                        password = newValue!;
                      });
                    },
                    obscureText: true,
                  ),
                  CheckboxListTile(
                    // 保存密码checkbox
                    dense: true,
                    contentPadding: const EdgeInsets.all(0.0),
                    controlAffinity: ListTileControlAffinity.leading,
                    title: Text(l10n.savePassword),
                    value: savePassword,
                    onChanged: (value) {
                      setState(() {
                        key.currentState!.save();
                        savePassword = !savePassword;
                      });
                    },
                  ),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Padding(
                        padding: EdgeInsets.all(AppStyles.mediumPadding),
                        child: ElevatedButton.icon(
                          style: AppStyles.defaultButtonStyle,
                          icon: Icon(Icons.arrow_back),
                          onPressed: () {
                            Navigator.pushReplacement(
                              context,
                              MaterialPageRoute(
                                builder: (context) => ServerSetting(),
                              ),
                            );
                          },
                          label: Text(l10n.selectServer),
                        ),
                      ),
                      Padding(
                        padding: EdgeInsets.all(AppStyles.mediumPadding),
                        child: ElevatedButton.icon(
                          style: AppStyles.defaultButtonStyle,
                          icon: Icon(Icons.login),
                          onPressed: () async {
                            key.currentState!.save(); // 保存表单信息
                            String? email, ocid;
                            if (account.contains('@')) {
                              // 判断邮箱/ocid登录
                              email = account;
                            } else {
                              ocid = account;
                            }
                            bool res = await ref
                                .read(authProvider.notifier)
                                .login(
                                  password: password,
                                  ocid: ocid,
                                  email: email,
                                );

                            if (res) {
                              final authState = ref.read(authProvider);
                              final accountId = authState.accountId!;

                              // 保存配置
                              var notifier = ref.read(configProvider.notifier);
                              notifier.setRecent(
                                account,
                                (savePassword ? password : ""),
                              );

                              // 创建私有数据库
                              privateDB = OurChatDatabase(accountId);
                              // 获取账户信息
                              await ref
                                  .read(
                                    ourChatAccountProvider(accountId).notifier,
                                  )
                                  .getAccountInfo();
                              // 更新头像URL
                              final avatarUrl = ref
                                  .read(ourChatAccountProvider(accountId))
                                  .avatarKey;
                              if (avatarUrl != null) {
                                notifier.setAvatarUrl(
                                  "http${ref.read(ourChatServerProvider).isTLS! ? 's' : ''}://${ref.read(ourChatServerProvider).host}:${ref.read(ourChatServerProvider).port.toString()}/v1/avatar?user_id=${accountId.toString()}&avatar_key=$avatarUrl",
                                );
                              }

                              ref
                                  .read(ourChatEventSystemProvider.notifier)
                                  .listenEvents();
                              // 应用状态已由AuthNotifier更新，无需额外更新
                              if (context.mounted) {
                                // 跳转主界面
                                Navigator.pop(context);
                              }
                            }
                          },
                          label: Text(l10n.login),
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
            Flexible(flex: 1, child: Container()),
          ],
        ),
      ),
    );
  }
}

// 注册
class Register extends ConsumerStatefulWidget {
  const Register({super.key});

  @override
  ConsumerState<Register> createState() => _RegisterState();
}

class _RegisterState extends ConsumerState<Register> {
  String email = "", password = "", username = "";
  bool showPassword = false, savePassword = false;
  @override
  Widget build(BuildContext context) {
    var key = GlobalKey<FormState>();
    return Form(
      key: key,
      child: Row(
        children: [
          Flexible(flex: 1, child: Container()),
          Flexible(
            flex: 3,
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                TextFormField(
                  // 用户名输入框
                  initialValue: username,
                  decoration: InputDecoration(label: Text(l10n.username)),
                  onSaved: (newValue) {
                    setState(() {
                      username = newValue!;
                    });
                  },
                ),
                TextFormField(
                  // 邮箱输入框
                  initialValue: email,
                  decoration: InputDecoration(label: Text(l10n.email)),
                  onSaved: (newValue) {
                    setState(() {
                      email = newValue!;
                    });
                  },
                ),
                TextFormField(
                  // 密码输入框
                  initialValue: password,
                  decoration: InputDecoration(label: Text(l10n.password)),
                  onSaved: (newValue) {
                    setState(() {
                      password = newValue!;
                    });
                  },
                  obscureText: !showPassword,
                ),
                CheckboxListTile(
                  // 显示密码checkbox
                  dense: true,
                  controlAffinity: ListTileControlAffinity.leading,
                  title: Text(l10n.show(l10n.password)),
                  value: showPassword,
                  onChanged: (value) {
                    setState(() {
                      key.currentState!.save();
                      showPassword = !showPassword;
                    });
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  controlAffinity: ListTileControlAffinity.leading,
                  title: Text(l10n.savePassword),
                  value: savePassword,
                  onChanged: (value) {
                    setState(() {
                      key.currentState!.save();
                      savePassword = !savePassword;
                    });
                  },
                ),
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Padding(
                      padding: EdgeInsets.all(AppStyles.mediumPadding),
                      child: ElevatedButton.icon(
                        style: AppStyles.defaultButtonStyle,
                        icon: Icon(Icons.arrow_back),
                        onPressed: () {
                          Navigator.pushReplacement(
                            context,
                            MaterialPageRoute(
                              builder: (context) => ServerSetting(),
                            ),
                          );
                        },
                        label: Text(l10n.selectServer),
                      ),
                    ),
                    Padding(
                      padding: EdgeInsets.all(AppStyles.mediumPadding),
                      child: ElevatedButton.icon(
                        style: AppStyles.defaultButtonStyle,
                        icon: Icon(Icons.app_registration),
                        onPressed: () async {
                          key.currentState!.save(); // 保存表单信息
                          bool res = await ref
                              .read(authProvider.notifier)
                              .register(
                                email: email,
                                password: password,
                                username: username,
                              );
                          if (res) {
                            final authState = ref.read(authProvider);
                            final accountId = authState.accountId!;

                            // 保存配置
                            var notifier = ref.watch(configProvider.notifier);
                            notifier.setRecent(
                              email,
                              (savePassword ? password : ""),
                            );

                            // 创建私有数据库
                            privateDB = OurChatDatabase(accountId);
                            // 获取账户信息
                            await ref
                                .read(
                                  ourChatAccountProvider(accountId).notifier,
                                )
                                .getAccountInfo();
                            // 更新头像URL
                            final avatarUrl = ref
                                .read(ourChatAccountProvider(accountId))
                                .avatarKey;
                            if (avatarUrl != null) {
                              notifier.setAvatarUrl(
                                "http${ref.read(ourChatServerProvider).isTLS! ? 's' : ''}://${ref.read(ourChatServerProvider).host}:${ref.read(ourChatServerProvider).port.toString()}/v1/avatar?user_id=${accountId.toString()}&avatar_key=$avatarUrl",
                              );
                            }

                            ref
                                .read(ourChatEventSystemProvider.notifier)
                                .listenEvents();
                            // 应用状态已由AuthNotifier更新，无需额外更新
                            if (context.mounted) {
                              // 注册成功后跳转到主页
                              Navigator.pop(context);
                            }
                          }
                        },
                        label: Text(l10n.register),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          Flexible(flex: 1, child: Container()),
        ],
      ),
    );
  }
}
