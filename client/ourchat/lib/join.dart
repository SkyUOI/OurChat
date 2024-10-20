import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:crypto/crypto.dart';
import 'dart:convert';
import 'main.dart';
import 'const.dart';
import 'setting.dart';

import 'package:flutter_gen/gen_l10n/app_localizations.dart';

class Join extends StatefulWidget {
  const Join({super.key});

  @override
  State<Join> createState() => _JoinState();
}

class _JoinState extends State<Join> {
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
    return ChangeNotifierProvider(
      create: (context) {
        var joinState = JoinState();
        joinState.ourchatAppState = ourchatAppState;
        return joinState;
      },
      child: Scaffold(
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
      ),
    );
  }
}

class JoinState extends ChangeNotifier {
  var account = "";
  var password = "";
  var nickname = "";
  var showPassword = false;
  var errorText = "";
  var page = 0; // 0: login, 1: register
  BuildContext? context;
  OurchatAppState? ourchatAppState;

  void setContext(BuildContext value) {
    context = value;
  }

  void setPage(var value) {
    page = value;
  }

  void setPassword(var value) {
    showPassword = value;
    notifyListeners();
  }

  void connectResponse(var data) {
    ourchatAppState!.unlisten(serverStatusMsgCode, connectResponse);
    if (data["status"] == operationSuccessfulStatusCode) {
      if (page == 0) {
        login();
      } else {
        register();
      }
    } else if (data["status"] == serverErrorStatusCode) {
      errorText = AppLocalizations.of(context!)!.serverError;
    } else if (data["status"] == underMaintenanceStatusCode) {
      errorText = AppLocalizations.of(context!)!.serverUnderMaintenance;
    } else {
      errorText = AppLocalizations.of(context!)!.unknownError;
    }
    notifyListeners();
  }

  bool checkTextField() {
    if (account.isEmpty || password.isEmpty) {
      errorText = AppLocalizations.of(context!)!.cantBeEmpty;
      notifyListeners();
      return false;
    }
    errorText = "";
    notifyListeners();
    return true;
  }

  void login() {
    ourchatAppState!.listen(loginResponseMsgCode, loginResponse);
    ourchatAppState!.connection!.send({
      "code": loginMsgCode,
      "login_type": !account.contains("@"),
      "account": account,
      "password": sha256.convert(utf8.encode(password)).toString(),
    });
  }

  void loginResponse(var data) {
    ourchatAppState!.unlisten(loginResponseMsgCode, loginResponse);
    if (data["status"] == operationSuccessfulStatusCode) {
      ourchatAppState!.toSomewhere(homeUi);
    } else if (data["status"] == serverErrorStatusCode) {
      errorText = AppLocalizations.of(context!)!.serverError;
    } else if (data["status"] == underMaintenanceStatusCode) {
      errorText = AppLocalizations.of(context!)!.serverUnderMaintenance;
    } else if (data["status"] == parameterErrorStatusCode) {
      errorText = AppLocalizations.of(context!)!.wrongAccountOrPassword;
    } else {
      errorText = AppLocalizations.of(context!)!.unknownError;
    }
    notifyListeners();
  }

  void initVerify() {
    ourchatAppState!.listen(initVerifyResponseMsgCode, initVerifyResponse);
    ourchatAppState!.connection!.send({
      "code": initVerifyMsgCode,
      "email": account,
    });
  }

  void initVerifyResponse(var data) {
    ourchatAppState!.unlisten(initVerifyResponseMsgCode, initVerifyResponse);
    if (data["status"] == operationSuccessfulStatusCode) {
      ourchatAppState!.unlisten(verifyResponseMsgCode, verifyResponse);
      errorText = AppLocalizations.of(context!)!.plzCheckYourEmail;
    } else if (data["status"] == serverErrorStatusCode) {
      errorText = AppLocalizations.of(context!)!.serverError;
    } else if (data["status"] == requestInfoDoesNotExistStatusCode) {
      errorText = AppLocalizations.of(context!)!.emailAddressUnreachable;
    } else if (data["status"] == accountRestrictionStatusCode) {
      errorText = AppLocalizations.of(context!)!.requestRefused;
    } else {
      errorText = AppLocalizations.of(context!)!.unknownError;
    }
    notifyListeners();
  }

  void verifyResponse(var data) {
    ourchatAppState!.unlisten(verifyResponseMsgCode, verifyResponse);
    if (data["status"] == operationSuccessfulStatusCode) {
      register();
    } else if (data["status"] == serverErrorStatusCode) {
      errorText = AppLocalizations.of(context!)!.serverError;
    } else if (data["status"] == underMaintenanceStatusCode) {
      errorText = AppLocalizations.of(context!)!.serverUnderMaintenance;
    } else if (data["status"] == timeoutStatusCode) {
      errorText = AppLocalizations.of(context!)!.verifyTimeout;
    } else {
      errorText = AppLocalizations.of(context!)!.unknownError;
    }
    notifyListeners();
  }

  void register() {
    ourchatAppState!.listen(registerResponseMsgCode, registerResponse);
    ourchatAppState!.connection!.send({
      "code": registerMsgCode,
      "email": account,
      "password": sha256.convert(utf8.encode(password)).toString(),
    });
  }

  void registerResponse(var data) {
    ourchatAppState!.unlisten(registerResponseMsgCode, registerResponse);
    if (data["status"] == operationSuccessfulStatusCode) {
      ourchatAppState!.toSomewhere(homeUi);
    } else if (data["status"] == serverErrorStatusCode) {
      errorText = AppLocalizations.of(context!)!.serverError;
    } else if (data["status"] == underMaintenanceStatusCode) {
      errorText = AppLocalizations.of(context!)!.serverUnderMaintenance;
    } else if (data["status"] == newInfoAlreadyExistsStatusCode) {
      errorText = AppLocalizations.of(context!)!.emailExists;
    } else if (data["status"] == parameterErrorStatusCode) {
      errorText = AppLocalizations.of(context!)!.verifyNotCompleted;
    } else {
      errorText = AppLocalizations.of(context!)!.unknownError;
    }
    notifyListeners();
  }
}

class Login extends StatelessWidget {
  const Login({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurchatAppState>();
    var joinState = context.watch<JoinState>();
    joinState.setPage(0);
    joinState.setContext(context);
    return Scaffold(
      body: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          TextField(
            decoration: InputDecoration(
                labelText:
                    "${AppLocalizations.of(context)!.email}/${AppLocalizations.of(context)!.ocid}"),
            controller: TextEditingController(text: joinState.account),
            onChanged: (value) {
              joinState.account = value;
            },
          ),
          TextField(
            decoration: InputDecoration(
              labelText: AppLocalizations.of(context)!.password,
            ),
            controller: TextEditingController(text: joinState.password),
            onChanged: (value) {
              joinState.password = value;
            },
            obscureText: !joinState.showPassword,
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              GestureDetector(
                child: Text(AppLocalizations.of(context)!.showPassword),
                onTap: () {
                  joinState.setPassword(!joinState.showPassword);
                },
              ),
              Checkbox(
                  value: joinState.showPassword,
                  onChanged: (value) {
                    joinState.setPassword(value!);
                  }),
            ],
          ),
          Container(
              margin: const EdgeInsets.only(top: 20),
              child: ElevatedButton(
                  onPressed: () {
                    if (!joinState.checkTextField()) {
                      return;
                    }
                    if (ourchatAppState.connection!.closed) {
                      ourchatAppState.listen(
                          serverStatusMsgCode, joinState.connectResponse);
                      ourchatAppState.connection!.connectToServer();
                    } else {
                      joinState.login();
                    }
                  },
                  child: Text(AppLocalizations.of(context)!.login))),
          Text(
            joinState.errorText,
            style: const TextStyle(color: Colors.red),
          ),
        ],
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
    var joinState = context.watch<JoinState>();
    var ourchatAppState = context.watch<OurchatAppState>();
    joinState.setContext(context);
    joinState.setPage(1);
    return Scaffold(
      body: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          TextField(
            decoration:
                InputDecoration(labelText: AppLocalizations.of(context)!.email),
            controller: TextEditingController(text: joinState.account),
            onChanged: (value) {
              joinState.account = value;
            },
          ),
          TextField(
            decoration: InputDecoration(
                labelText: AppLocalizations.of(context)!.nickname),
            controller: TextEditingController(text: joinState.nickname),
            onChanged: (value) {
              joinState.nickname = value;
            },
          ),
          TextField(
            decoration: InputDecoration(
              labelText: AppLocalizations.of(context)!.password,
            ),
            controller: TextEditingController(text: joinState.password),
            onChanged: (value) {
              joinState.password = value;
            },
            obscureText: !joinState.showPassword,
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              GestureDetector(
                child: Text(AppLocalizations.of(context)!.showPassword),
                onTap: () {
                  joinState.setPassword(!joinState.showPassword);
                },
              ),
              Checkbox(
                  value: joinState.showPassword,
                  onChanged: (value) {
                    joinState.setPassword(value!);
                  }),
            ],
          ),
          Container(
              margin: const EdgeInsets.only(top: 20),
              child: ElevatedButton(
                  onPressed: () {
                    if (!joinState.checkTextField()) {
                      return;
                    }
                    if (ourchatAppState.connection!.closed) {
                      ourchatAppState.listen(
                          serverStatusMsgCode, joinState.connectResponse);
                      ourchatAppState.connection!.connectToServer();
                    } else {
                      joinState.register();
                    }
                  },
                  child: Text(AppLocalizations.of(context)!.register))),
          Text(
            joinState.errorText,
            style: const TextStyle(color: Colors.red),
          ),
        ],
      ),
    );
  }
}
