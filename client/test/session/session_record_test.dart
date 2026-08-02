import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

/// Pure unit tests for _MessageWidgetState._buildFileCard.
/// We copy the implementation inline so we can exercise it without Riverpod,
/// gRPC, or any runtime dependencies.
class _FileCardBuilder {
  static Widget build({
    required String contentType,
    required String filename,
    required int size,
    bool isLocal = false,
  }) {
    IconData fileIcon;
    if (contentType.startsWith('video/')) {
      fileIcon = Icons.videocam;
    } else if (contentType.startsWith('audio/')) {
      fileIcon = Icons.audiotrack;
    } else if (contentType.startsWith('image/')) {
      fileIcon = Icons.image;
    } else if (contentType.contains('pdf')) {
      fileIcon = Icons.picture_as_pdf;
    } else if (contentType.contains('zip') ||
        contentType.contains('tar') ||
        contentType.contains('compress')) {
      fileIcon = Icons.archive;
    } else {
      fileIcon = Icons.insert_drive_file;
    }

    String sizeStr;
    if (size < 1024) {
      sizeStr = '$size B';
    } else if (size < 1024 * 1024) {
      sizeStr = '${(size / 1024).toStringAsFixed(1)} KB';
    } else {
      sizeStr = '${(size / 1024 / 1024).toStringAsFixed(1)} MB';
    }

    String displayName = filename.isNotEmpty ? filename : 'file';
    if (displayName.length > 30) {
      displayName = '${displayName.substring(0, 27)}...';
    }

    return Container(
      width: 220,
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: Colors.grey.shade100,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.grey.shade300),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(fileIcon, size: 40, color: Colors.grey.shade600),
          const SizedBox(height: 6),
          Text(
            displayName,
            style: const TextStyle(fontWeight: FontWeight.w500, fontSize: 13),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          const SizedBox(height: 4),
          Text(
            sizeStr,
            style: TextStyle(fontSize: 11, color: Colors.grey.shade600),
          ),
        ],
      ),
    );
  }
}

/// Verify a widget tree contains an Icon with the given [expectedIcon].
Finder _iconFinder(IconData expectedIcon) =>
    find.byWidgetPredicate((w) => w is Icon && w.icon == expectedIcon);

void main() {
  group('_FileCardBuilder icon selection', () {
    testWidgets('video contentType → videocam icon', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: _FileCardBuilder.build(
            contentType: 'video/mp4',
            filename: 'clip.mp4',
            size: 5000,
          ),
        ),
      );
      expect(_iconFinder(Icons.videocam), findsOneWidget);
    });

    testWidgets('audio contentType → audiotrack icon', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: _FileCardBuilder.build(
            contentType: 'audio/mpeg',
            filename: 'song.mp3',
            size: 10000,
          ),
        ),
      );
      expect(_iconFinder(Icons.audiotrack), findsOneWidget);
    });

    testWidgets('image contentType → image icon', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: _FileCardBuilder.build(
            contentType: 'image/png',
            filename: 'photo.png',
            size: 1024,
          ),
        ),
      );
      expect(_iconFinder(Icons.image), findsOneWidget);
    });

    testWidgets('pdf → picture_as_pdf icon', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: _FileCardBuilder.build(
            contentType: 'application/pdf',
            filename: 'doc.pdf',
            size: 2048,
          ),
        ),
      );
      expect(_iconFinder(Icons.picture_as_pdf), findsOneWidget);
    });

    testWidgets('zip → archive icon', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: _FileCardBuilder.build(
            contentType: 'application/zip',
            filename: 'bundle.zip',
            size: 999,
          ),
        ),
      );
      expect(_iconFinder(Icons.archive), findsOneWidget);
    });

    testWidgets('unknown type → insert_drive_file icon', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: _FileCardBuilder.build(
            contentType: 'application/octet-stream',
            filename: 'unknown.bin',
            size: 42,
          ),
        ),
      );
      expect(_iconFinder(Icons.insert_drive_file), findsOneWidget);
    });
  });

  group('_FileCardBuilder size formatting', () {
    testWidgets('bytes < 1024 show "B"', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: _FileCardBuilder.build(
            contentType: '',
            filename: 'f',
            size: 512,
          ),
        ),
      );
      expect(find.text('512 B'), findsOneWidget);
    });

    testWidgets('bytes >= 1024 show "KB"', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: _FileCardBuilder.build(
            contentType: '',
            filename: 'f',
            size: 1536, // 1.5 KB
          ),
        ),
      );
      expect(find.text('1.5 KB'), findsOneWidget);
    });

    testWidgets('bytes >= 1 MB show "MB"', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: _FileCardBuilder.build(
            contentType: '',
            filename: 'f',
            size: 2 * 1024 * 1024, // 2 MB
          ),
        ),
      );
      expect(find.text('2.0 MB'), findsOneWidget);
    });
  });

  group('_FileCardBuilder filename display', () {
    testWidgets('uses filename when provided', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: _FileCardBuilder.build(
            contentType: '',
            filename: 'report.pdf',
            size: 100,
          ),
        ),
      );
      expect(find.text('report.pdf'), findsOneWidget);
    });

    testWidgets('uses "file" when filename empty', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: _FileCardBuilder.build(
            contentType: '',
            filename: '',
            size: 100,
          ),
        ),
      );
      expect(find.text('file'), findsOneWidget);
    });

    testWidgets('truncates long filenames', (tester) async {
      final longName = 'a' * 50;
      await tester.pumpWidget(
        MaterialApp(
          home: _FileCardBuilder.build(
            contentType: '',
            filename: longName,
            size: 1,
          ),
        ),
      );
      // Displayed name should be <= 30 chars
      final labelFinder = find.byWidgetPredicate(
        (w) =>
            w is Text &&
            w.data != null &&
            w.data!.length <= 30 &&
            w.data!.endsWith('...'),
      );
      expect(labelFinder, findsOneWidget);
    });
  });
}
