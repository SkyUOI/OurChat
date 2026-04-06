import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';

/// 应用程序样式常量
class AppStyles {
  // 间距
  static const double smallPadding = 5.0;
  static const double defaultPadding = 8.0;
  static const double mediumPadding = 10.0;
  static const double largePadding = 16.0;

  // 圆角
  static const double defaultBorderRadius = 10.0;

  // 字体大小
  static const double smallFontSize = 14.0;
  static const double defaultFontSize = 16.0;
  static const double titleFontSize = 20.0;
  static const double largeFontSize = 25.0;

  // 图标大小
  static const double smallIconSize = 20.0;
  static const double defaultIconSize = 24.0;

  // 头像尺寸
  static const double smallAvatarSize = 20.0;
  static const double defaultAvatarSize = 40.0;
  static const double largeAvatarSize = 100.0;

  // 卡片样式
  static BoxDecoration cardDecoration(BuildContext context) {
    return BoxDecoration(
      color: Theme.of(context).cardColor,
      borderRadius: BorderRadius.circular(defaultBorderRadius),
      boxShadow: [
        BoxShadow(
          color: Colors.black.withValues(alpha: 0.1),
          blurRadius: 4,
          offset: const Offset(0, 2),
        ),
      ],
    );
  }

  // 按钮样式
  static ButtonStyle defaultButtonStyle = ButtonStyle(
    shape: WidgetStateProperty.all(
      RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(defaultBorderRadius),
      ),
    ),
  );
}

class UserAvatar extends StatelessWidget {
  final String imageUrl;
  final double size;
  final VoidCallback? onTap;
  final bool showEditIcon;

  const UserAvatar({
    Key? key,
    required this.imageUrl,
    this.size = AppStyles.defaultAvatarSize,
    this.onTap,
    this.showEditIcon = false,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Stack(
        children: [
          SizedBox(
            width: size,
            height: size,
            child: ClipRRect(
              borderRadius: BorderRadius.circular(size / 4),
              child: CachedNetworkImage(
                imageUrl: imageUrl,
                fit: BoxFit.cover,
                errorWidget: (context, url, error) {
                  return Icon(
                    Icons.account_circle,
                    size: size,
                    color: Theme.of(context).disabledColor,
                  );
                },
              ),
            ),
          ),
          if (showEditIcon)
            Positioned(
              right: 0,
              bottom: 0,
              child: Container(
                width: size * 0.3,
                height: size * 0.3,
                decoration: BoxDecoration(
                  color: Theme.of(context).cardColor,
                  borderRadius: BorderRadius.only(
                    topLeft: Radius.circular(AppStyles.smallPadding),
                  ),
                ),
                child: Icon(Icons.edit, size: AppStyles.smallIconSize),
              ),
            ),
        ],
      ),
    );
  }
}
