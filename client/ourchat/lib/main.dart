import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'login.dart';

void main() {
  runApp(const MainApp());
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
    final Widget page;
    if (currentIndex == 0) {
      page = const Login();
    } else if (currentIndex == 1) {
      page = const Register();
    } else {
      page = const Placeholder();
    }
    return ChangeNotifierProvider(
      create: (context) => LoginStatus(),
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
