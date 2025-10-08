import 'package:flutter/material.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/core/database.dart';
import 'core/account.dart';
import 'package:ourchat/home.dart';
import 'package:provider/provider.dart';

// Auth界面
class Auth extends StatelessWidget {
  const Auth({super.key});
  @override
  Widget build(BuildContext context) {
    var l10n = AppLocalizations.of(context)!;
    return Scaffold(
      body: DefaultTabController(
        length: 2,
        child: Column(
          children: [
            TabBar(
              tabs: [
                Tab(text: l10n.login),
                Tab(text: l10n.register),
              ],
            ),
            const Expanded(child: TabBarView(children: [Login(), Register()])),
          ],
        ),
      ),
    );
  }
}

class Login extends StatefulWidget {
  const Login({super.key});

  @override
  State<Login> createState() => _LoginState();
}

class _LoginState extends State<Login> {
  String account = "", password = "", avatarUrl = "";
  bool savePassword = false, inited = false;
  @override
  Widget build(BuildContext context) {
    var key = GlobalKey<FormState>();
    var ourchatAppState = context.watch<OurChatAppState>();
    if (!inited) {
      account = ourchatAppState.config["recent_account"];
      password = ourchatAppState.config["recent_password"];
      avatarUrl = ourchatAppState.config["recent_avatar_url"];
      if (password.isNotEmpty) savePassword = true;
      inited = true;
    }
    var l10n = AppLocalizations.of(context)!;
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
                    Padding(
                      padding: EdgeInsets.all(10.0),
                      child: SizedBox(
                          height: 100.0,
                          width: 100.0,
                          child: (avatarUrl.isEmpty
                              ? Image.asset("assets/images/logo.png")
                              : Image(image: NetworkImage(avatarUrl)))),
                    ),
                    TextFormField(
                      // 账号输入框
                      initialValue: account,
                      decoration: InputDecoration(
                          label: Text("${l10n.ocid}/${l10n.email}")),
                      onSaved: (newValue) {
                        setState(() {
                          account = newValue!;
                        });
                      },
                    ),
                    TextFormField(
                      // 密码输入框
                      initialValue: password,
                      decoration: InputDecoration(
                        label: Text(l10n.password),
                      ),
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
                        }),
                    Padding(
                      padding: EdgeInsets.all(AppStyles.mediumPadding),
                      child: ElevatedButton.icon(
                          style: AppStyles.defaultButtonStyle,
                          icon: Icon(Icons.login),
                          onPressed: () async {
                            key.currentState!.save(); // 保存表单信息
                            // 创建ocAccount对象并登录
                            OurChatAccount ocAccount =
                                OurChatAccount(ourchatAppState);
                            String? email, ocid;
                            if (account.contains('@')) {
                              // 判断邮箱/ocid登录
                              email = account;
                            } else {
                              ocid = account;
                            }
                            var res =
                                await ocAccount.login(password, ocid, email);
                            var code = res.$1, message = res.$2;
                            if (code == okStatusCode) {
                              ourchatAppState.config["recent_account"] =
                                  account;
                              ourchatAppState.config["recent_password"] =
                                  (savePassword ? password : "");
                              ourchatAppState.config["recent_avatar_url"] =
                                  ocAccount.avatarUrl();
                              ourchatAppState.config.saveConfig();
                              ourchatAppState.thisAccount = ocAccount;
                              ourchatAppState.privateDB =
                                  OurChatDatabase(ocAccount.id);
                              ourchatAppState.eventSystem =
                                  OurChatEventSystem(ourchatAppState);
                              await ourchatAppState.thisAccount!
                                  .getAccountInfo();
                              ourchatAppState.eventSystem!.listenEvents();
                              ourchatAppState.update();
                              if (context.mounted) {
                                // 跳转主界面
                                Navigator.pop(context);
                                Navigator.push(context, MaterialPageRoute(
                                  builder: (context) {
                                    return const Scaffold(
                                      body: Home(),
                                    );
                                  },
                                ));
                              }
                            } else {
                              logger.w("login fail: code $code");
                              // 处理报错
                              if (context.mounted) {
                                showResultMessage(context, code, message,
                                    notFoundStatus: l10n.notFound(l10n.user),
                                    invalidArgumentStatus: l10n.internalError,
                                    unauthenticatedStatus:
                                        l10n.incorrectPassword);
                              }
                            }
                          },
                          label: Text(l10n.login)),
                    )
                  ],
                )),
            Flexible(flex: 1, child: Container())
          ],
        ));
  }
}

// 注册
class Register extends StatefulWidget {
  const Register({super.key});

  @override
  State<Register> createState() => _RegisterState();
}

class _RegisterState extends State<Register> {
  String email = "", password = "", username = "";
  bool showPassword = false;
  @override
  Widget build(BuildContext context) {
    var key = GlobalKey<FormState>();
    var ourchatAppState = context.watch<OurChatAppState>();
    var l10n = AppLocalizations.of(context)!;
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
                      decoration: InputDecoration(
                        label: Text(l10n.password),
                      ),
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
                        contentPadding: const EdgeInsets.all(0.0),
                        controlAffinity: ListTileControlAffinity.leading,
                        title: Text(l10n.showPassword),
                        value: showPassword,
                        onChanged: (value) {
                          setState(() {
                            key.currentState!.save();
                            showPassword = !showPassword;
                          });
                        }),
                    Padding(
                      padding: EdgeInsets.all(AppStyles.mediumPadding),
                      child: ElevatedButton.icon(
                          style: AppStyles.defaultButtonStyle,
                          icon: Icon(Icons.app_registration),
                          onPressed: () async {
                            key.currentState!.save(); // 保存表单信息
                            // 创建ocAccount对象并注册
                            OurChatAccount ocAccount =
                                OurChatAccount(ourchatAppState);
                            var res = await ocAccount.register(
                                password, username, email);
                            var code = res.$1, message = res.$2;
                            if (code == okStatusCode) {
                              // 注册成功
                              ourchatAppState.thisAccount = ocAccount;
                              ourchatAppState.privateDB =
                                  OurChatDatabase(ocAccount.id);
                              ourchatAppState.eventSystem =
                                  OurChatEventSystem(ourchatAppState);
                              await ourchatAppState.thisAccount!
                                  .getAccountInfo();

                              ourchatAppState.eventSystem!.listenEvents();
                              ourchatAppState.update();
                              if (context.mounted) {
                                // 注册成功后跳转到主页
                                Navigator.pop(context);
                                Navigator.push(context, MaterialPageRoute(
                                  builder: (context) {
                                    return const Scaffold(
                                      body: Home(),
                                    );
                                  },
                                ));
                              }
                            } else {
                              logger.w("register fail: code $code");
                              // 处理报错
                              if (context.mounted) {
                                showResultMessage(context, code, message,
                                    alreadyExistsStatus: l10n.emailExists,
                                    invalidArgumentStatus: {
                                      "Password Is Not Strong Enough":
                                          l10n.passwordIsNotStrongEnough,
                                      "Username Is Invalid":
                                          l10n.invalid(l10n.username),
                                      "Email Address Is Invalid":
                                          l10n.invalid(l10n.email),
                                    });
                              }
                            }
                          },
                          label: Text(l10n.register)),
                    ),
                  ],
                )),
            Flexible(flex: 1, child: Container())
          ],
        ));
  }
}
