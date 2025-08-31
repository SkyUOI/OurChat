import 'package:fixnum/fixnum.dart';
import 'package:ourchat/google/protobuf/timestamp.pb.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/l10n/app_localizations.dart';

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
  dynamic message = AppLocalizations.of(context)!.unknownError;
  switch (code) {
    case okStatusCode:
      if (okStatus != null) {
        message = okStatus;
      }
      message = AppLocalizations.of(context)!.succeeded;
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
        message = AppLocalizations.of(context)!.serverError;
      }
      break;
    case unavailableStatusCode:
      if (unavailableStatus != null) {
        message = unavailableStatus;
      } else {
        message = AppLocalizations.of(context)!.serverStatusUnderMaintenance;
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
  if (message is String) {
    ScaffoldMessenger.of(context)
        .showSnackBar(SnackBar(content: Text(message)));
  } else if (message is Map) {
    ScaffoldMessenger.of(context)
        .showSnackBar(SnackBar(content: Text(message[errorMessage])));
  }
}
