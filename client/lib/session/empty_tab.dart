import 'package:flutter/material.dart';

class EmptyTab extends StatelessWidget {
  const EmptyTab({super.key});

  @override
  Widget build(BuildContext context) {
    return Center(child: Image.asset("assets/images/logo.png", width: 300));
  }
}
