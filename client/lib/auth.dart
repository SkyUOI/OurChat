import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:ourchat/const.dart';
import 'package:ourchat/main.dart';
import 'ourchat/ourchat_account.dart';
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
  bool showPassword = false;
  @override
  Widget build(BuildContext context) {
    var key = GlobalKey<FormState>();
    var ourchatAppState = context.watch<OurchatAppState>();
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
                            // 创建ocAccount对象并登录
                            OurchatAccount ocAccount =
                                OurchatAccount(ourchatAppState.server!);
                            String? email, ocid;
                            if (account.contains('@')) {
                              // 判断邮箱/ocid登录
                              email = account;
                            } else {
                              ocid = account;
                            }
                            var res =
                                await ocAccount.login(password, ocid, email);
                            if (res == okStatusCode) {
                              ourchatAppState.thisAccount = ocAccount;
                              ourchatAppState.thisAccount!.getAccountInfo();
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
                              // 处理报错
                              setState(() {
                                switch (res) {
                                  case internalStatusCode:
                                    // 服务端内部错误
                                    ScaffoldMessenger.of(context).showSnackBar(
                                        SnackBar(
                                            content: Text(
                                                AppLocalizations.of(context)!
                                                    .serverError)));
                                    break;
                                  case unavailableStatusCode:
                                    // 服务端维护中
                                    ScaffoldMessenger.of(context).showSnackBar(
                                        SnackBar(
                                            content: Text(AppLocalizations.of(
                                                    context)!
                                                .serverStatusUnderMaintenance)));
                                    break;
                                  case notFoundStatusCode:
                                    // 用户不存在
                                    ScaffoldMessenger.of(context).showSnackBar(
                                        SnackBar(
                                            content: Text(
                                                AppLocalizations.of(context)!
                                                    .userNotFound)));
                                    break;
                                  case invalidArgumentStatusCode:
                                    // 缺少AuthType 理论上不会出现该报错
                                    ScaffoldMessenger.of(context).showSnackBar(
                                        SnackBar(
                                            content: Text(
                                                AppLocalizations.of(context)!
                                                    .internalError)));
                                    break;
                                  case unauthenticatedStatusCode:
                                    // 密码错误
                                    ScaffoldMessenger.of(context).showSnackBar(
                                        SnackBar(
                                            content: Text(
                                                AppLocalizations.of(context)!
                                                    .incorrectPassword)));
                                    break;
                                  default:
                                    // 未知错误
                                    ScaffoldMessenger.of(context).showSnackBar(
                                        SnackBar(
                                            content: Text(
                                                AppLocalizations.of(context)!
                                                    .unknownError)));
                                    break;
                                }
                              });
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
    var ourchatAppState = context.watch<OurchatAppState>();
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
                            OurchatAccount ocAccount =
                                OurchatAccount(ourchatAppState.server!);
                            var res = await ocAccount.register(
                                password, username, email);
                            if (res == okStatusCode) {
                              // 注册成功
                              ourchatAppState.thisAccount = ocAccount;
                              ourchatAppState.thisAccount!.getAccountInfo();
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
                              // 处理报错
                              setState(() {
                                switch (res) {
                                  case internalStatusCode:
                                    // 服务端内部错误
                                    ScaffoldMessenger.of(context)
                                        .showSnackBar(SnackBar(
                                      content: Text(
                                          AppLocalizations.of(context)!
                                              .serverError),
                                    ));
                                    break;
                                  case unavailableStatusCode:
                                    // 服务端维护中
                                    ScaffoldMessenger.of(context)
                                        .showSnackBar(SnackBar(
                                      content: Text(
                                          AppLocalizations.of(context)!
                                              .serverStatusUnderMaintenance),
                                    ));
                                    break;
                                  case alreadyExistsStatusCode:
                                    // 邮箱已存在
                                    ScaffoldMessenger.of(context)
                                        .showSnackBar(SnackBar(
                                      content: Text(
                                          AppLocalizations.of(context)!
                                              .emailExists),
                                    ));
                                    break;
                                  default:
                                    // 未知错误
                                    ScaffoldMessenger.of(context)
                                        .showSnackBar(SnackBar(
                                      content: Text(
                                          AppLocalizations.of(context)!
                                              .unknownError),
                                    ));
                                    break;
                                }
                              });
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
