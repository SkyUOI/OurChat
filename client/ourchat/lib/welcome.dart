import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart';
import 'package:ourchat/service/auth/authorize/v1/authorize.pb.dart';
import 'package:ourchat/service/auth/register/v1/register.pb.dart';
import 'package:provider/provider.dart';
import 'main.dart';
import 'const.dart';
import 'setting.dart';
import 'package:crypto/crypto.dart';

import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:ourchat/service/auth/v1/auth.pbgrpc.dart';

class Welcome extends StatefulWidget {
  const Welcome({super.key});

  @override
  State<Welcome> createState() => _WelcomeState();
}

class _WelcomeState extends State<Welcome> {
  var currentIndex = 0;
  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurchatAppState>();
    final Widget page;
    if (currentIndex == 0) {
      page = const Login();
    } else if (currentIndex == 1) {
      page = const Register();
    } else {
      page = const Setting();
    }
    return Scaffold(
      body: Column(
        children: [
          SafeArea(
            child: BottomNavigationBar(
              elevation: 0.0,
              items: [
                BottomNavigationBarItem(
                    icon: const Icon(Icons.login),
                    label: AppLocalizations.of(context)!.login),
                BottomNavigationBarItem(
                    icon: const Icon(Icons.person_add),
                    label: AppLocalizations.of(context)!.register),
                BottomNavigationBarItem(
                    icon: const Icon(Icons.settings),
                    label: AppLocalizations.of(context)!.setting)
              ],
              currentIndex: currentIndex,
              onTap: (index) {
                setState(() {
                  currentIndex = index;
                });
              },
            ),
          ),
          Expanded(
              child: Align(
                  alignment: Alignment.center,
                  child: AspectRatio(
                      aspectRatio: 9 / 16,
                      child: Padding(
                          padding:
                              const EdgeInsets.only(left: 20.0, right: 20.0),
                          child: page))))
        ],
      ),
    );
  }
}

class Login extends StatefulWidget {
  const Login({
    super.key,
  });

  @override
  State<Login> createState() => _LoginState();
}

class _LoginState extends State<Login> {
  String errorText = "";
  String? account, password;
  bool showPassword = false;
  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurchatAppState>();
    var key = GlobalKey<FormState>();
    return Scaffold(
      body: Form(
        key: key,
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextFormField(
              decoration: InputDecoration(
                  labelText:
                      "${AppLocalizations.of(context)!.email}/${AppLocalizations.of(context)!.ocid}"),
              controller: TextEditingController(text: account),
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return AppLocalizations.of(context)!.cantBeEmpty;
                }
                return null;
              },
              onChanged: (value) {
                account = value;
              },
            ),
            TextFormField(
              decoration: InputDecoration(
                labelText: AppLocalizations.of(context)!.password,
              ),
              controller: TextEditingController(text: password),
              obscureText: showPassword,
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return AppLocalizations.of(context)!.cantBeEmpty;
                }
                return null;
              },
              onChanged: (value) {
                password = value;
              },
            ),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                GestureDetector(
                  child: Text(AppLocalizations.of(context)!.showPassword),
                  onTap: () {
                    setState(() {
                      showPassword = !showPassword;
                    });
                  },
                ),
                Checkbox(
                    value: showPassword,
                    onChanged: (value) {
                      setState(() {
                        showPassword = !value!;
                      });
                    }),
              ],
            ),
            Container(
                margin: const EdgeInsets.only(top: 20),
                child: ElevatedButton(
                    onPressed: () async {
                      if (key.currentState!.validate()) {
                        ourchatAppState.connection!.setAddress(
                            ourchatAppState.config!.data!["server_address"],
                            int.parse(
                                ourchatAppState.config!.data!["ws_port"]));
                        var result =
                            await ourchatAppState.connection!.connectToServer();
                        if (result["status"] == cannotConnectServer) {
                          setState(() {
                            errorText = AppLocalizations.of(context)!
                                .cantConnectToServer(result["msg"]);
                          });
                        } else if (result["status"] == okStatusCode) {
                          String encodedPassword = sha256
                              .convert(ascii.encode(password!))
                              .toString();
                          AuthServiceClient stub = AuthServiceClient(
                              ourchatAppState.connection!.channel!);
                          try {
                            print(222);
                            AuthResponse? res;
                            if (account!.contains('@')) {
                              var res = await stub.auth(AuthRequest(
                                  email: account, password: encodedPassword));
                            } else {
                              var res = await stub.auth(AuthRequest(
                                  ocid: account, password: encodedPassword));
                            }
                            print(111);
                            print(res!);
                          } on GrpcError catch (e) {
                            if (e.code == unauthenticatedStatusCode) {
                              setState(() {
                                errorText = AppLocalizations.of(context)!
                                    .wrongAccountOrPassword;
                                return;
                              });
                            }
                            print(e);
                          }
                        } else if (result["status"] ==
                            failedPreconditionStatusCode) {
                          setState(() {
                            errorText = AppLocalizations.of(context)!
                                .cantConnectToServer(
                                    AppLocalizations.of(context)!
                                        .requestRefused);
                          });
                        }
                      }
                    },
                    child: Text(AppLocalizations.of(context)!.login))),
            Text(
              errorText,
              style: const TextStyle(color: Colors.red),
            ),
          ],
        ),
      ),
    );
  }
}

class Register extends StatefulWidget {
  const Register({
    super.key,
  });

  @override
  State<Register> createState() => _RegisterState();
}

class _RegisterState extends State<Register> {
  String errorText = "";
  String? account, password, username;
  bool showPassword = false;
  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurchatAppState>();
    var key = GlobalKey<FormState>();
    return Scaffold(
      body: Form(
        key: key,
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextFormField(
              decoration: InputDecoration(
                  labelText: AppLocalizations.of(context)!.username),
              controller: TextEditingController(text: username),
              onChanged: (value) {
                username = value;
              },
            ),
            TextFormField(
              decoration: InputDecoration(
                  labelText: AppLocalizations.of(context)!.email),
              controller: TextEditingController(text: account),
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return AppLocalizations.of(context)!.cantBeEmpty;
                }
                return null;
              },
              onChanged: (value) {
                account = value;
              },
            ),
            TextFormField(
              decoration: InputDecoration(
                labelText: AppLocalizations.of(context)!.password,
              ),
              controller: TextEditingController(text: password),
              obscureText: showPassword,
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return AppLocalizations.of(context)!.cantBeEmpty;
                }
                return null;
              },
              onChanged: (value) {
                password = value;
              },
            ),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                GestureDetector(
                  child: Text(AppLocalizations.of(context)!.showPassword),
                  onTap: () {
                    setState(() {
                      showPassword = !showPassword;
                    });
                  },
                ),
                Checkbox(
                    value: showPassword,
                    onChanged: (value) {
                      setState(() {
                        showPassword = !value!;
                      });
                    }),
              ],
            ),
            Container(
                margin: const EdgeInsets.only(top: 20),
                child: ElevatedButton(
                    onPressed: () async {
                      if (key.currentState!.validate()) {
                        ourchatAppState.connection!.setAddress(
                            ourchatAppState.config!.data!["server_address"],
                            int.parse(
                                ourchatAppState.config!.data!["ws_port"]));
                        var result =
                            await ourchatAppState.connection!.connectToServer();
                        if (result["status"] == cannotConnectServer) {
                          setState(() {
                            errorText = AppLocalizations.of(context)!
                                .cantConnectToServer(result["msg"]);
                          });
                        } else if (result["status"] == okStatusCode) {
                          String encodedPassword = sha256
                              .convert(ascii.encode(password!))
                              .toString();
                          AuthServiceClient stub = AuthServiceClient(
                              ourchatAppState.connection!.channel!);
                          try {
                            RegisterResponse? res;
                            if (username!.isEmpty) {
                              var res = await stub.register(RegisterRequest(
                                  email: account, password: encodedPassword));
                            } else {
                              var res = await stub.register(RegisterRequest(
                                  email: account,
                                  password: encodedPassword,
                                  name: username));
                            }
                            print(res);
                          } on GrpcError catch (e) {
                            print(e);
                          }
                        } else if (result["status"] ==
                            failedPreconditionStatusCode) {
                          setState(() {
                            errorText = AppLocalizations.of(context)!
                                .cantConnectToServer(
                                    AppLocalizations.of(context)!
                                        .requestRefused);
                          });
                        }
                      }
                    },
                    child: Text(AppLocalizations.of(context)!.register))),
            Text(
              errorText,
              style: const TextStyle(color: Colors.red),
            ),
          ],
        ),
      ),
    );
  }
}
