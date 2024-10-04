import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:crypto/crypto.dart';
import 'dart:convert';
import 'main.dart';
import 'const.dart';

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
      page = const Placeholder();
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
                items: const [
                  BottomNavigationBarItem(
                      icon: Icon(Icons.login), label: "Login"),
                  BottomNavigationBarItem(
                      icon: Icon(Icons.person_add), label: "Register"),
                  BottomNavigationBarItem(
                      icon: Icon(Icons.settings), label: "Setting")
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
                    child: AspectRatio(aspectRatio: 9 / 16, child: page)))
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
  OurchatAppState? ourchatAppState;

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
      errorText = "Server Error";
    } else if (data["status"] == underMaintenanceStatusCode) {
      errorText = "Server is under Maintenance";
    } else {
      errorText = "Unknown Error";
    }
    notifyListeners();
  }

  bool checkTextField() {
    if (account.isEmpty || password.isEmpty) {
      errorText = "account/password can't be empty";
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
      errorText = "Server Error";
    } else if (data["status"] == underMaintenanceStatusCode) {
      errorText = "Server is under Maintenance";
    } else if (data["status"] == parameterErrorStatusCode) {
      errorText = "wrong account/password";
    } else {
      errorText = "Unknown Error";
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
      errorText = "Please check your email";
    } else if (data["status"] == serverErrorStatusCode) {
      errorText = "Server Error";
    } else if (data["status"] == requestInfoDoesNotExistStatusCode) {
      errorText = "Email address is unreachable";
    } else if (data["status"] == accountRestrictionStatusCode) {
      errorText = "Request refused";
    } else {
      errorText = "Unknown Error";
    }
    notifyListeners();
  }

  void verifyResponse(var data) {
    ourchatAppState!.unlisten(verifyResponseMsgCode, verifyResponse);
    if (data["status"] == operationSuccessfulStatusCode) {
      register();
    } else if (data["status"] == serverErrorStatusCode) {
      errorText = "Server Error";
    } else if (data["status"] == underMaintenanceStatusCode) {
      errorText = "Server is under Maintenance";
    } else if (data["status"] == timeoutStatusCode) {
      errorText = "Verification timeout";
    } else {
      errorText = "Unknown Error";
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
      errorText = "Server Error";
    } else if (data["status"] == underMaintenanceStatusCode) {
      errorText = "Server is under Maintenance";
    } else if (data["status"] == newInfoAlreadyExistsStatusCode) {
      errorText = "Email already exists";
    } else if (data["status"] == parameterErrorStatusCode) {
      errorText = "Verification not completed";
    } else {
      errorText = "Unknown Error";
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
    joinState.page = 0;
    return Scaffold(
        body: Padding(
      padding: const EdgeInsets.only(left: 20.0, right: 20.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          TextField(
            decoration: const InputDecoration(labelText: "Email/OCID"),
            controller: TextEditingController(text: joinState.account),
            onChanged: (value) {
              joinState.account = value;
            },
          ),
          TextField(
            decoration: const InputDecoration(
              labelText: "Password",
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
                child: const Text("Show Password"),
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
                  child: const Text("Login"))),
          Text(
            joinState.errorText,
            style: const TextStyle(color: Colors.red),
          ),
        ],
      ),
    ));
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
    joinState.page = 1;
    return Scaffold(
        body: Padding(
      padding: const EdgeInsets.only(left: 20.0, right: 20.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          TextField(
            decoration: const InputDecoration(labelText: "Email"),
            controller: TextEditingController(text: joinState.account),
            onChanged: (value) {
              joinState.account = value;
            },
          ),
          TextField(
            decoration: const InputDecoration(labelText: "Nickname"),
            controller: TextEditingController(text: joinState.nickname),
            onChanged: (value) {
              joinState.nickname = value;
            },
          ),
          TextField(
            decoration: const InputDecoration(
              labelText: "Password",
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
                child: const Text("Show Password"),
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
                  child: const Text("Register"))),
          Text(
            joinState.errorText,
            style: const TextStyle(color: Colors.red),
          ),
        ],
      ),
    ));
  }
}
