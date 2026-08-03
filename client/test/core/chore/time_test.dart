import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/chore/time.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';

/// Tests for `OurChatTime`, the bridge between gRPC `Timestamp` and Dart
/// `DateTime`. These conversions are used when storing and rendering every
/// message, so regressions here corrupt message ordering/display.
void main() {
  group('OurChatTime round trips', () {
    test('datetime -> timestamp -> datetime is stable (second precision)', () {
      final original = DateTime.utc(2026, 8, 2, 12, 30, 45);
      final t = OurChatTime.fromDatetime(original);
      // Recompose through the timestamp representation.
      final back = OurChatTime.fromTimestamp(t.timestamp);
      expect(back.datetime.difference(original).inSeconds, 0);
    });

    test('timestamp -> datetime uses seconds-since-epoch', () {
      // 2026-08-02T12:30:45Z
      const epochSeconds = 1785213045;
      final t = OurChatTime.fromTimestamp(
        Timestamp(seconds: Int64(epochSeconds)),
      );
      expect(t.datetime.microsecondsSinceEpoch, epochSeconds * 1000000);
    });

    test('datetime -> timestamp records correct epoch seconds', () {
      final dt = DateTime.utc(2025, 1, 1, 0, 0, 0);
      final t = OurChatTime.fromDatetime(dt);
      expect(t.timestamp.seconds.toInt(), dt.millisecondsSinceEpoch ~/ 1000);
    });
  });

  group('OurChatTime equality', () {
    test('two times within the same second are equal', () {
      final a = OurChatTime.fromDatetime(DateTime.utc(2026, 1, 1, 0, 0, 0));
      final b = OurChatTime.fromDatetime(DateTime.utc(2026, 1, 1, 0, 0, 0));
      expect(a == b, isTrue);
      expect(a.hashCode, b.hashCode);
    });

    test('times in different seconds are not equal', () {
      final a = OurChatTime.fromDatetime(DateTime.utc(2026, 1, 1, 0, 0, 0));
      final b = OurChatTime.fromDatetime(DateTime.utc(2026, 1, 1, 0, 0, 5));
      expect(a == b, isFalse);
    });

    test('comparing to a non-OurChatTime returns false', () {
      final a = OurChatTime.fromDatetime(DateTime.utc(2026, 1, 1));
      // ignore: unrelated_type_equality_checks
      expect(a == 'not a time', isFalse);
    });
  });

  group('OurChatTime edge cases', () {
    test('epoch zero', () {
      final t = OurChatTime.fromTimestamp(Timestamp(seconds: Int64.ZERO));
      expect(t.datetime.microsecondsSinceEpoch, 0);
    });

    test('sub-second fractions below 500ms round down in the timestamp', () {
      // The class rounds to whole seconds when building a Timestamp.
      final withMs = OurChatTime.fromDatetime(
        DateTime.utc(2026, 1, 1, 0, 0, 0, 100),
      );
      final onSecond = OurChatTime.fromDatetime(
        DateTime.utc(2026, 1, 1, 0, 0, 0, 0),
      );
      expect(withMs.timestamp.seconds, onSecond.timestamp.seconds);
    });

    test('500ms rounds up to the next second', () {
      final t = OurChatTime.fromDatetime(
        DateTime.utc(2026, 1, 1, 0, 0, 0, 500),
      );
      final nextSecond = OurChatTime.fromDatetime(
        DateTime.utc(2026, 1, 1, 0, 0, 1, 0),
      );
      expect(t.timestamp.seconds, nextSecond.timestamp.seconds);
    });

    test('equality is exact on the datetime (microsecond precision)', () {
      // Note: == compares the full-precision datetime, not the rounded
      // timestamp. Two OurChatTimes built from distinct datetimes are never
      // equal even if their rounded timestamps coincide.
      final a = OurChatTime.fromDatetime(
        DateTime.utc(2026, 1, 1, 0, 0, 0, 100),
      );
      final b = OurChatTime.fromDatetime(DateTime.utc(2026, 1, 1, 0, 0, 0, 0));
      expect(a == b, isFalse);
    });
  });
}
