import 'dart:async';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_cache_manager/flutter_cache_manager.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:hashlib/hashlib.dart';
import 'package:image_compression/image_compression.dart';
import 'package:ourchat/core/chore/grpc_utils.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/download/v1/download.pb.dart';
import 'package:ourchat/service/ourchat/upload/v1/upload.pb.dart';

Future<UploadResponse> uploadStreaming(
  OurChatServer server,
  Uint8List rawData,
  bool autoClean, {
  Int64? sessionId,
  bool compress = true,
  String? contentType,
}) async {
  var stub = server.newStub();
  StreamController<UploadRequest> controller =
      StreamController<UploadRequest>();
  var call = safeRequest(stub.upload, controller.stream, (GrpcError e) {
    showResultMessage(
      e.code,
      e.message,
      invalidArgumentStatus: "${l10n.internalError}(${e.message})",
      resourceExhaustedStatus: l10n.storageSpaceFull,
    );
  }, rethrowError: true);
  Uint8List biData = rawData;
  if (compress) {
    biData = (await compressInQueue(
      ImageFileConfiguration(
        input: ImageFile(
          filePath: "",
          rawBytes: rawData,
          contentType: contentType,
        ),
      ),
    )).rawBytes;
    logger.i(
      "upload: original size=${rawData.lengthInBytes}, compressed size=${biData.lengthInBytes}",
    );
  }
  controller.add(
    UploadRequest(
      metadata: Header(
        hash: sha3_256.convert(biData.toList()).bytes,
        size: Int64.parseInt(biData.length.toString()),
        autoClean: autoClean,
        sessionId: sessionId,
      ),
    ),
  );
  int chunkSize = 1024 * 128;
  for (int i = 0; i < biData.lengthInBytes; i += chunkSize) {
    logger.i(
      "upload: sending chunk ${i ~/ chunkSize + 1} of ${(biData.lengthInBytes / chunkSize).ceil()}",
    );
    controller.add(
      UploadRequest(
        content: biData
            .sublist(
              i,
              biData.lengthInBytes > i + chunkSize
                  ? i + chunkSize
                  : biData.lengthInBytes,
            )
            .toList(),
      ),
    );
    logger.i("finish");
  }
  controller.close();
  return await call;
}

Future<UploadResponse> upload(
  OurChatServer server,
  Uint8List rawData,
  bool autoClean, {
  Int64? sessionId,
  bool compress = true,
  String? contentType,
}) async {
  if (kIsWeb) {
    return await uploadChunked(
      server,
      rawData,
      autoClean,
      sessionId: sessionId,
      compress: compress,
      contentType: contentType,
    );
  } else {
    return await uploadStreaming(
      server,
      rawData,
      autoClean,
      sessionId: sessionId,
      compress: compress,
      contentType: contentType,
    );
  }
}

/// Chunked upload function for gRPC-web compatibility
/// Uses multiple unary RPC calls instead of streaming
Future<UploadResponse> uploadChunked(
  OurChatServer server,
  Uint8List rawData,
  bool autoClean, {
  Int64? sessionId,
  bool compress = true,
  String? contentType,
}) async {
  var stub = server.newStub();

  // Compress if needed
  Uint8List biData = rawData;
  if (compress) {
    biData = (await compressInQueue(
      ImageFileConfiguration(
        input: ImageFile(
          filePath: "",
          rawBytes: rawData,
          contentType: contentType,
        ),
      ),
    )).rawBytes;
    logger.i(
      "uploadChunked: original size=${rawData.lengthInBytes}, compressed size=${biData.lengthInBytes}",
    );
  }

  // Calculate hash
  var hash = sha3_256.convert(biData.toList()).bytes;

  // Start upload session
  var startResp = await safeRequest(
    stub.startUpload,
    StartUploadRequest(
      hash: hash,
      size: Int64.parseInt(biData.length.toString()),
      autoClean: autoClean,
      sessionId: sessionId,
    ),
    (GrpcError e) {
      showResultMessage(
        e.code,
        e.message,
        resourceExhaustedStatus: l10n.storageSpaceFull,
      );
    },
    rethrowError: true,
  );

  String uploadId = startResp.uploadId;
  int chunkSize = startResp.chunkSize.toInt();

  // Upload chunks sequentially
  int totalChunks = (biData.length / chunkSize).ceil();
  for (int i = 0; i < totalChunks; i++) {
    int start = i * chunkSize;
    int end = (start + chunkSize < biData.length)
        ? start + chunkSize
        : biData.length;
    var chunk = biData.sublist(start, end);

    var chunkResp = await safeRequest(
      stub.uploadChunk,
      UploadChunkRequest(
        uploadId: uploadId,
        chunkData: chunk,
        chunkId: Int64.parseInt(i.toString()),
      ),
      (GrpcError e) {
        showResultMessage(
          e.code,
          e.message,
          invalidArgumentStatus: "${l10n.internalError}(${e.message})",
        );
      },
      rethrowError: true,
    );

    logger.i(
      "Uploaded chunk ${i + 1}/$totalChunks (${chunkResp.bytesReceived}/${biData.length} bytes)",
    );
  }

  // Complete upload
  var completeResp = await safeRequest(
    stub.completeUpload,
    CompleteUploadRequest(uploadId: uploadId),
    (GrpcError e) {
      showResultMessage(
        e.code,
        e.message,
        invalidArgumentStatus: "${l10n.internalError}(${e.message})",
      );
    },
    rethrowError: true,
  );

  return UploadResponse(key: completeResp.key);
}

/// Cancel an ongoing upload
Future<void> cancelUpload(OurChatServer server, String uploadId) async {
  var stub = server.newStub();

  await safeRequest(
    stub.cancelUpload,
    CancelUploadRequest(uploadId: uploadId),
    (GrpcError e) {
      logger.e("Failed to cancel upload: $e");
    },
  );
}

Future<Uint8List> getOurChatFile(WidgetRef ref, String key) async {
  try {
    var manager = DefaultCacheManager();
    var server = ref.watch(ourChatServerProvider);
    FileInfo? cache = await manager.getFileFromCache(
      "${server.host}:${server.port}/$key",
    );
    if (cache != null) {
      return await cache.file.readAsBytes();
    }
    var stub = server.newStub();
    var res =
        await safeRequest(stub.download, DownloadRequest(key: key), (
              GrpcError e,
            ) {
              showResultMessage(e.code, e.message);
            })
            as ResponseStream<DownloadResponse>;
    List<int> data = [];
    for (DownloadResponse piece in await res.toList()) {
      data.addAll(piece.data);
    }

    manager.putFile(
      "${server.host}:${server.port}/$key",
      Uint8List.fromList(data),
      key: "${server.host}:${server.port}/$key",
    );
    return Uint8List.fromList(data);
  } catch (e) {
    logger.e("getOurChatFile(key:$key) error:$e");
    rethrow;
  }
}
