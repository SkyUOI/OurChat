import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

void main() {
  runApp(const MainApp());
}

class OurChatAppStatus extends ChangeNotifier {
  var number = 123;
}

class MainApp extends StatefulWidget {
  const MainApp({super.key});

  @override
  State<MainApp> createState() => _MainAppState();
}

class _MainAppState extends State<MainApp> {
  var currentIndex = 0;
  @override
  Widget build(BuildContext context) {
    var page;
    if (currentIndex == 0) {
      page = const Login();
    } else if (currentIndex == 1) {
      page = const Register();
    } else {
      page = const Placeholder();
    }
    return ChangeNotifierProvider(
      create: (context) => OurChatAppStatus(),
      child: MaterialApp(
        home: Scaffold(
          body: Column(
            children: [
              SafeArea(
                child: BottomNavigationBar(
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
              Expanded(child: page)
            ],
          ),
        ),
        theme: ThemeData(
            useMaterial3: true,
            colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue)),
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
  var showPassword = false;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        body: Padding(
      padding: const EdgeInsets.only(left: 20.0, right: 20.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const TextField(
            decoration: InputDecoration(labelText: "Email/OCID"),
          ),
          TextField(
            decoration: const InputDecoration(
              labelText: "Password",
            ),
            obscureText: !showPassword,
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              GestureDetector(
                child: const Text("Show Password"),
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
                      showPassword = value!;
                    });
                  }),
            ],
          ),
          Container(
              margin: const EdgeInsets.only(top: 20),
              child:
                  ElevatedButton(onPressed: () {}, child: const Text("Login")))
        ],
      ),
    ));
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
  var showPassword = false;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        body: Padding(
      padding: const EdgeInsets.only(left: 20.0, right: 20.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const TextField(
            decoration: InputDecoration(labelText: "Email"),
          ),
          const TextField(
            decoration: InputDecoration(labelText: "Nickname"),
          ),
          TextField(
            decoration: const InputDecoration(
              labelText: "Password",
            ),
            obscureText: !showPassword,
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              GestureDetector(
                child: const Text("Show Password"),
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
                      showPassword = value!;
                    });
                  }),
            ],
          ),
          Container(
              margin: const EdgeInsets.only(top: 20),
              child: ElevatedButton(
                  onPressed: () {}, child: const Text("Register")))
        ],
      ),
    ));
  }
}
