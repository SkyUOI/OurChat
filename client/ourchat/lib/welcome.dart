import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart';
// import 'package:ourchat/ourchat/ourchat_account.dart';
import 'package:ourchat/service/auth/authorize/v1/authorize.pb.dart';
import 'package:ourchat/service/auth/register/v1/register.pb.dart';
import 'package:provider/provider.dart';
import 'main.dart';
import 'const.dart';
import 'setting.dart';
import 'package:crypto/crypto.dart';
import 'package:logger/logger.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:ourchat/service/auth/v1/auth.pbgrpc.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';

class WelcomeData extends ChangeNotifier {
  String errorText = "";
  String? account, password, username;
  bool showPassword = false;

  void setErrorText(String et) {
    errorText = et;
    notifyListeners();
  }

  void changePasswordVisibility({bool? value}) {
    if (value == null) {
      showPassword = !showPassword;
    } else {
      showPassword = value;
    }
    notifyListeners();
  }
}

class Welcome extends StatefulWidget {
  const Welcome({super.key});

  @override
  State<Welcome> createState() => _WelcomeState();
}

class _WelcomeState extends State<Welcome> {
  var currentIndex = 0;
  @override
  Widget build(BuildContext context) {
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
                          child: ChangeNotifierProvider(
                              create: (_) => WelcomeData(), child: page)))))
        ],
      ),
    );
  }
}

class Login extends StatelessWidget {
  const Login({
    super.key,
  });
  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurchatAppState>();
    var welcomeData = context.watch<WelcomeData>();
    var key = GlobalKey<FormState>();
    Logger logger = Logger();
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
              controller: TextEditingController(text: welcomeData.account),
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return AppLocalizations.of(context)!.cantBeEmpty;
                }
                return null;
              },
              onChanged: (value) {
                welcomeData.account = value;
              },
            ),
            TextFormField(
              decoration: InputDecoration(
                labelText: AppLocalizations.of(context)!.password,
              ),
              controller: TextEditingController(text: welcomeData.password),
              obscureText: !welcomeData.showPassword,
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return AppLocalizations.of(context)!.cantBeEmpty;
                }
                return null;
              },
              onChanged: (value) {
                welcomeData.password = value;
              },
            ),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                GestureDetector(
                  child: Text(AppLocalizations.of(context)!.showPassword),
                  onTap: () {
                    welcomeData.changePasswordVisibility();
                  },
                ),
                Checkbox(
                    value: welcomeData.showPassword,
                    onChanged: (value) {
                      welcomeData.changePasswordVisibility(value: value);
                    }),
              ],
            ),
            Container(
                margin: const EdgeInsets.only(top: 20),
                child: ElevatedButton(
                    onPressed: () async {
                      if (key.currentState!.validate()) {
                        var channel = ClientChannel(
                            ourchatAppState.config!.data!["server_address"],
                            port: int.parse(
                                ourchatAppState.config!.data!["ws_port"]),
                            options: const ChannelOptions(
                                credentials: ChannelCredentials.insecure()));
                        final stub = BasicServiceClient(channel);
                        try {
                          await stub.getServerInfo(GetServerInfoRequest());
                        } on GrpcError catch (e) {
                          if (e.code == failedPreconditionStatusCode) {
                            if (!context.mounted) return;
                            welcomeData.setErrorText(
                                AppLocalizations.of(context)!.requestRefused);
                          }
                        }
                        (e) {
                          if (!context.mounted) return;
                          logger.e("get server info error: $e");
                          welcomeData.setErrorText(AppLocalizations.of(context)!
                              .cantConnectToServer(e.toString()));
                          return;
                        };
                        if (!context.mounted) return;
                        String encodedPassword = sha256
                            .convert(ascii.encode(welcomeData.password!))
                            .toString();
                        AuthServiceClient authStub = AuthServiceClient(channel);
                        try {
                          if (welcomeData.account!.contains('@')) {
                            var res = await authStub.auth(AuthRequest(
                                email: welcomeData.account,
                                password: encodedPassword));
                            welcomeData.setErrorText("");
                            res;
                          } else {
                            var res = await authStub.auth(AuthRequest(
                                ocid: welcomeData.account,
                                password: encodedPassword));
                            welcomeData.setErrorText("");
                            res;
                          }
                        } on GrpcError catch (e) {
                          if (e.code == unauthenticatedStatusCode ||
                              e.code == notFoundStatusCode) {
                            if (!context.mounted) return;
                            welcomeData.setErrorText(
                                AppLocalizations.of(context)!
                                    .wrongAccountOrPassword);
                            return;
                          }
                          logger.e("Login error: $e");
                          if (!context.mounted) return;
                          welcomeData.setErrorText(e.toString());
                        }
                      }
                    },
                    child: Text(AppLocalizations.of(context)!.login))),
            Text(
              welcomeData.errorText,
              style: const TextStyle(color: Colors.red),
            ),
          ],
        ),
      ),
    );
  }
}

class Register extends StatelessWidget {
  const Register({
    super.key,
  });
  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurchatAppState>();
    var welcomeData = context.watch<WelcomeData>();
    var key = GlobalKey<FormState>();
    Logger logger = Logger();
    return Scaffold(
      body: Form(
        key: key,
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextFormField(
              decoration: InputDecoration(
                  labelText: AppLocalizations.of(context)!.username),
              controller: TextEditingController(text: welcomeData.username),
              onChanged: (value) {
                welcomeData.username = value;
              },
            ),
            TextFormField(
              decoration: InputDecoration(
                  labelText: AppLocalizations.of(context)!.email),
              controller: TextEditingController(text: welcomeData.account),
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return AppLocalizations.of(context)!.cantBeEmpty;
                }
                return null;
              },
              onChanged: (value) {
                welcomeData.account = value;
              },
            ),
            TextFormField(
              decoration: InputDecoration(
                labelText: AppLocalizations.of(context)!.password,
              ),
              controller: TextEditingController(text: welcomeData.password),
              obscureText: !welcomeData.showPassword,
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return AppLocalizations.of(context)!.cantBeEmpty;
                }
                return null;
              },
              onChanged: (value) {
                welcomeData.password = value;
              },
            ),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                GestureDetector(
                  child: Text(AppLocalizations.of(context)!.showPassword),
                  onTap: () {
                    welcomeData.changePasswordVisibility();
                  },
                ),
                Checkbox(
                    value: welcomeData.showPassword,
                    onChanged: (value) {
                      welcomeData.changePasswordVisibility(value: value);
                    }),
              ],
            ),
            Container(
                margin: const EdgeInsets.only(top: 20),
                child: ElevatedButton(
                    onPressed: () async {
                      if (key.currentState!.validate()) {
                        var channel = ClientChannel(
                            ourchatAppState.config!.data!["server_address"],
                            port: int.parse(
                                ourchatAppState.config!.data!["ws_port"]),
                            options: const ChannelOptions(
                                credentials: ChannelCredentials.insecure()));
                        final stub = BasicServiceClient(channel);
                        try {
                          await stub.getServerInfo(GetServerInfoRequest());
                        } on GrpcError catch (e) {
                          logger.e("get server info error: $e");
                          if (e.code == failedPreconditionStatusCode) {
                            if (!context.mounted) return;
                            welcomeData.setErrorText(
                                AppLocalizations.of(context)!.requestRefused);
                          }
                        } catch (e) {
                          logger.e("get server info error: $e");
                          if (!context.mounted) return;
                          welcomeData.setErrorText(AppLocalizations.of(context)!
                              .cantConnectToServer(e.toString()));
                          return;
                        }
                        if (!context.mounted) return;
                        String encodedPassword = sha256
                            .convert(ascii.encode(welcomeData.password!))
                            .toString();
                        AuthServiceClient authStub = AuthServiceClient(channel);
                        try {
                          if (welcomeData.username!.isEmpty) {
                            var res = await authStub.register(RegisterRequest(
                                email: welcomeData.account,
                                password: encodedPassword));
                            res;
                          } else {
                            var res = await authStub.register(RegisterRequest(
                                email: welcomeData.account,
                                password: encodedPassword,
                                name: welcomeData.username));
                            res;
                          }
                        } on GrpcError catch (e) {
                          logger.e("Register error: $e");
                        }
                      }
                    },
                    child: Text(AppLocalizations.of(context)!.register))),
            Text(
              welcomeData.errorText,
              style: const TextStyle(color: Colors.red),
            ),
          ],
        ),
      ),
    );
  }
}
