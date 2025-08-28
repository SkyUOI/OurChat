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
    return Scaffold(
      body: DefaultTabController(
        length: 2,
        child: Column(
          children: [
            TabBar(
              tabs: [
                Tab(text: AppLocalizations.of(context)!.login),
                Tab(text: AppLocalizations.of(context)!.register),
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
  String account = "", password = "";
  bool savePassword = false, inited = false;
  @override
  Widget build(BuildContext context) {
    var key = GlobalKey<FormState>();
    var ourchatAppState = context.watch<OurChatAppState>();
    if (!inited) {
      account = ourchatAppState.config["recent_account"];
      password = ourchatAppState.config["recent_password"];
      if (password.isNotEmpty) savePassword = true;
      inited = true;
    }
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
                    const Padding(
                      padding: EdgeInsets.all(10.0),
                      child: SizedBox(
                          height: 100.0, width: 100.0, child: Placeholder()),
                    ),
                    TextFormField(
                      // 账号输入框
                      initialValue: account,
                      decoration: InputDecoration(
                          label: Text(
                              "${AppLocalizations.of(context)!.ocid}/${AppLocalizations.of(context)!.email}")),
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
                        label: Text(AppLocalizations.of(context)!.password),
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
                        title: Text(AppLocalizations.of(context)!.savePassword),
                        value: savePassword,
                        onChanged: (value) {
                          setState(() {
                            key.currentState!.save();
                            savePassword = !savePassword;
                          });
                        }),
                    Padding(
                      padding: const EdgeInsets.all(10.0),
                      child: ElevatedButton(
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
                                showErrorMessage(context, code, message,
                                    notFoundStatus:
                                        AppLocalizations.of(context)!.notFound(
                                            AppLocalizations.of(context)!.user),
                                    invalidArgumentStatus:
                                        AppLocalizations.of(context)!
                                            .internalError,
                                    unauthenticatedStatus:
                                        AppLocalizations.of(context)!
                                            .incorrectPassword);
                              }
                            }
                          },
                          child: Text(AppLocalizations.of(context)!.login)),
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
                      decoration: InputDecoration(
                          label: Text(AppLocalizations.of(context)!.username)),
                      onSaved: (newValue) {
                        setState(() {
                          username = newValue!;
                        });
                      },
                    ),
                    TextFormField(
                      // 邮箱输入框
                      initialValue: email,
                      decoration: InputDecoration(
                          label: Text(AppLocalizations.of(context)!.email)),
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
                        label: Text(AppLocalizations.of(context)!.password),
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
                        title: Text(AppLocalizations.of(context)!.showPassword),
                        value: showPassword,
                        onChanged: (value) {
                          setState(() {
                            key.currentState!.save();
                            showPassword = !showPassword;
                          });
                        }),
                    Padding(
                      padding: const EdgeInsets.all(10.0),
                      child: ElevatedButton(
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
                                showErrorMessage(context, code, message,
                                    alreadyExistsStatus:
                                        AppLocalizations.of(context)!
                                            .emailExists,
                                    invalidArgumentStatus: {
                                      "Password Is Not Strong Enough":
                                          AppLocalizations.of(context)!
                                              .passwordIsNotStrongEnough,
                                      "Username Is Invalid":
                                          AppLocalizations.of(context)!.invalid(
                                              AppLocalizations.of(context)!
                                                  .username),
                                      "Email Address Is Invalid":
                                          AppLocalizations.of(context)!.invalid(
                                              AppLocalizations.of(context)!
                                                  .email),
                                    });
                              }
                            }
                          },
                          child: Text(AppLocalizations.of(context)!.register)),
                    ),
                  ],
                )),
            Flexible(flex: 1, child: Container())
          ],
        ));
  }
}
