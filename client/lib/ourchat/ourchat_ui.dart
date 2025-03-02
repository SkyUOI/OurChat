import 'package:flutter/material.dart';

Widget cardWithPadding(Widget child, {double padding = 5.0}) {
  return Card(
    child: Padding(
      padding: EdgeInsets.all(padding),
      child: child,
    ),
  );
}
