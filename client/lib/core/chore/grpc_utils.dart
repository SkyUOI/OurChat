import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/main.dart';

void showResultMessage(
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
  dynamic message = l10n.unknownError;
  switch (code) {
    case okStatusCode:
      message = l10n.succeeded;
      if (okStatus != null) {
        message = okStatus;
      }
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
      break;
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
      break;
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
      rootScaffoldMessengerKey.currentState!.showSnackBar(
        SnackBar(content: Text(message)),
      );
    } else if (message is Map) {
      rootScaffoldMessengerKey.currentState!.showSnackBar(
        SnackBar(content: Text(message[errorMessage])),
      );
    }
  } catch (e) {
    logger.w("showResultMessage error: $e");
  }
}

Future safeRequest(
  Function func,
  var args,
  Function onError, {
  bool rethrowError = false,
}) async {
  logger.d("safeRequest called $func with args: $args");
  bool retryFlag = false;
  while (true) {
    try {
      var res = await func(args);
      if (retryFlag) {
        logger.i("Request succeeded after retry");
      }
      return res;
    } on GrpcError catch (e) {
      if (e.code == resourceExhaustedStatusCode &&
          e.message == "HTTP connection completed with 429 instead of 200") {
        retryFlag = true;
        logger.w("Rate limit exceeded, sleeping for a while");
        await Future.delayed(Duration(milliseconds: 500));
      } else {
        logger.w("GrpcError caught: $e");
        onError(e);
        if (rethrowError) {
          rethrow;
        }
        return;
      }
    } catch (e) {
      logger.w("Error caught: $e");
      if (rethrowError) {
        rethrow;
      }
    }
  }
}
