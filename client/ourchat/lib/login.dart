import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class LoginStatus extends ChangeNotifier {
  var account = "";
  var password = "";
  var nickname = "";
  var showPassword = false;

  void setPassword(var value) {
    showPassword = value;
    notifyListeners();
  }
}

class Login extends StatelessWidget {
  const Login({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    var loginStatus = context.watch<LoginStatus>();
    return Scaffold(
        body: Padding(
      padding: const EdgeInsets.only(left: 20.0, right: 20.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          TextField(
            decoration: const InputDecoration(labelText: "Email/OCID"),
            controller: TextEditingController(text: loginStatus.account),
            onChanged: (value) {
              loginStatus.account = value;
            },
          ),
          TextField(
            decoration: const InputDecoration(
              labelText: "Password",
            ),
            controller: TextEditingController(text: loginStatus.password),
            onChanged: (value) {
              loginStatus.password = value;
            },
            obscureText: !loginStatus.showPassword,
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              GestureDetector(
                child: const Text("Show Password"),
                onTap: () {
                  loginStatus.setPassword(!loginStatus.showPassword);
                },
              ),
              Checkbox(
                  value: loginStatus.showPassword,
                  onChanged: (value) {
                    loginStatus.setPassword(value!);
                  }),
            ],
          ),
          Container(
              margin: const EdgeInsets.only(top: 20),
              child: ElevatedButton(
                  onPressed: () {
                    print(loginStatus.account);
                  },
                  child: const Text("Login")))
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
    var loginStatus = context.watch<LoginStatus>();
    return Scaffold(
        body: Padding(
      padding: const EdgeInsets.only(left: 20.0, right: 20.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          TextField(
            decoration: const InputDecoration(labelText: "Email"),
            controller: TextEditingController(text: loginStatus.account),
            onChanged: (value) {
              loginStatus.account = value;
            },
          ),
          TextField(
            decoration: const InputDecoration(labelText: "Nickname"),
            controller: TextEditingController(text: loginStatus.nickname),
            onChanged: (value) {
              loginStatus.nickname = value;
            },
          ),
          TextField(
            decoration: const InputDecoration(
              labelText: "Password",
            ),
            controller: TextEditingController(text: loginStatus.password),
            onChanged: (value) {
              loginStatus.password = value;
            },
            obscureText: !loginStatus.showPassword,
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              GestureDetector(
                child: const Text("Show Password"),
                onTap: () {
                  loginStatus.setPassword(!loginStatus.showPassword);
                },
              ),
              Checkbox(
                  value: loginStatus.showPassword,
                  onChanged: (value) {
                    loginStatus.setPassword(value!);
                  }),
            ],
          ),
          Container(
              margin: const EdgeInsets.only(top: 20),
              child: ElevatedButton(
                  onPressed: () {
                    print(loginStatus.account);
                  },
                  child: const Text("Login")))
        ],
      ),
    ));
  }
}
