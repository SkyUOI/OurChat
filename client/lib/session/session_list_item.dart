import 'package:flutter/material.dart';

class SessionListItem extends StatelessWidget {
  const SessionListItem({
    super.key,
    required this.avatar,
    required this.name,
    required this.onPressed,
  });

  final Function onPressed;
  final Widget avatar;
  final String name;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      // 显示匹配账号
      height: 50.0,
      child: Padding(
        padding: const EdgeInsets.only(top: 5.0),
        child: ElevatedButton(
          style: ButtonStyle(
            shape: WidgetStateProperty.all(
              RoundedRectangleBorder(borderRadius: BorderRadius.circular(10.0)),
            ),
          ),
          onPressed: () => onPressed(),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              SizedBox(width: 40.0, height: 40.0, child: avatar),
              Text(name),
            ],
          ),
        ),
      ),
    );
  }
}
