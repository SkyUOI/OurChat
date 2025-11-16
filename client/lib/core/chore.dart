import 'package:fixnum/fixnum.dart';
import 'package:grpc/grpc.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/google/protobuf/timestamp.pb.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:cached_network_image/cached_network_image.dart';
import 'package:ourchat/main.dart';

class OurChatTime {
  /*
  该类用于grpc的timestamp与datetime两种时间类型之间的转换
  为了避免数据库的精度问题，转换所得的datetime并不准确
   */
  Timestamp? inputTimestamp;
  DateTime? inputDatetime;
  late Timestamp timestamp;
  late DateTime datetime;
  OurChatTime({this.inputTimestamp, this.inputDatetime}) {
    if (inputTimestamp != null) {
      timestamp = inputTimestamp!;
      toDatetime();
    } else {
      datetime = inputDatetime!;
      toTimestamp();
    }
  }

  void toTimestamp() {
    Int64 seconds = Int64.parseInt(
        (datetime.microsecondsSinceEpoch / 1000000).round().toString());
    // print(datetime.microsecondsSinceEpoch);
    // print("=>timestamp$seconds,$nanos");
    timestamp = Timestamp(seconds: seconds);
  }

  void toDatetime() {
    var seconds = timestamp.seconds;
    // print(timestamp);
    // print("=>datetime${seconds.toInt() * 1000000 + nanos}");
    datetime = DateTime.fromMicrosecondsSinceEpoch(seconds.toInt() * 1000000);
  }

  @override
  bool operator ==(Object other) {
    if (other is OurChatTime) {
      return datetime.difference(other.datetime).inMicroseconds == 0;
    }
    return false;
  }

  @override
  int get hashCode => timestamp.hashCode;
}

void showResultMessage(
  BuildContext context,
  int code,
  String? errorMessage, {
  dynamic okStatus,
  dynamic cancelledStatus,
  dynamic unknownStatus,
  dynamic invalidArgumentStatus,
  dynamic deadlineExceededStatus,
  dynamic notFoundStatus,
  dynamic alreadyExistsStatus,
  dynamic permissionDeniedStatus,
  dynamic resourceExhaustedStatus,
  dynamic failedPreconditionStatus,
  dynamic abortedStatus,
  dynamic outOfRangeStatus,
  dynamic unimplementedStatus,
  dynamic internalStatus,
  dynamic unavailableStatus,
  dynamic dataLossStatus,
  dynamic unauthenticatedStatus,
}) {
  var l10n = AppLocalizations.of(context)!;
  dynamic message = l10n.unknownError;
  switch (code) {
    case okStatusCode:
      if (okStatus != null) {
        message = okStatus;
      }
      message = l10n.succeeded;
      break;
    case cancelledStatusCode:
      if (cancelledStatus != null) {
        message = cancelledStatus;
      }
      break;
    case unknownStatusCode:
      if (unknownStatus != null) {
        message = unknownStatus;
      }
      break;
    case invalidArgumentStatusCode:
      if (invalidArgumentStatus != null) {
        message = invalidArgumentStatus;
      }
      break;
    case deadlineExceededStatusCode:
      if (deadlineExceededStatus != null) {
        message = deadlineExceededStatus;
      }
      break;
    case notFoundStatusCode:
      if (notFoundStatus != null) {
        message = notFoundStatus;
      }
      break;
    case alreadyExistsStatusCode:
      if (alreadyExistsStatus != null) {
        message = alreadyExistsStatus;
      }
      break;
    case permissionDeniedStatusCode:
      if (permissionDeniedStatus != null) {
        message = permissionDeniedStatus;
      }
      break;
    case resourceExhaustedStatusCode:
      if (resourceExhaustedStatus != null) {
        message = resourceExhaustedStatus;
      }
      break;
    case failedPreconditionStatusCode:
      if (failedPreconditionStatus != null) {
        message = failedPreconditionStatus;
      }
      break;
    case abortedStatusCode:
      if (abortedStatus != null) {
        message = abortedStatus;
      }
    case outOfRangeStatusCode:
      if (outOfRangeStatus != null) {
        message = outOfRangeStatus;
      }
      break;
    case unimplementedStatusCode:
      if (unimplementedStatus != null) {
        message = unimplementedStatus;
      }
      break;
    case internalStatusCode:
      if (internalStatus != null) {
        message = internalStatus;
      } else {
        message = l10n.serverError;
      }
      break;
    case unavailableStatusCode:
      if (unavailableStatus != null) {
        message = unavailableStatus;
      } else {
        message = l10n.serverStatusUnderMaintenance;
      }
      break;
    case dataLossStatusCode:
      if (dataLossStatus != null) {
        message = dataLossStatus;
      }
    case unauthenticatedStatusCode:
      if (unauthenticatedStatus != null) {
        message = unauthenticatedStatus;
      }
      break;
    default:
      break;
  }
  try {
    if (message is String && message.isNotEmpty) {
      rootScaffoldMessengerKey.currentState!
          .showSnackBar(SnackBar(content: Text(message)));
    } else if (message is Map) {
      rootScaffoldMessengerKey.currentState!
          .showSnackBar(SnackBar(content: Text(message[errorMessage])));
    }
  } catch (e) {
    logger.w("showResultMessage error:$e");
  }
}

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
                child: Icon(
                  Icons.edit,
                  size: AppStyles.smallIconSize,
                ),
              ),
            ),
        ],
      ),
    );
  }
}

Future safeRequest(Function func, var args) async {
  bool retryFlag = false;
  while (true) {
    try {
      var res = await func(args);
      if (retryFlag) {
        logger.i("Request succeeded after retry");
      }
      return res;
    } on GrpcError catch (e) {
      if (e.code == resourceExhaustedStatusCode) {
        retryFlag = true;
        logger.w("Rate limit exceeded, sleeping for a while");
        await Future.delayed(Duration(milliseconds: 500));
      } else {
        rethrow;
      }
    }
  }
}
