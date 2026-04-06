import 'package:fixnum/fixnum.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';

class OurChatTime {
  /*
  该类用于grpc的timestamp与datetime两种时间类型之间的转换
  为了避免数据库的精度问题，转换所得的datetime并不准确
   */
  Timestamp? inputTimestamp;
  DateTime? inputDatetime;
  late Timestamp timestamp;
  late DateTime datetime;

  OurChatTime.fromTimestamp(Timestamp ts) {
    timestamp = ts;
    toDatetime();
  }

  OurChatTime.fromDatetime(DateTime dt) {
    datetime = dt;
    toTimestamp();
  }

  void toTimestamp() {
    Int64 seconds = Int64.parseInt(
      (datetime.microsecondsSinceEpoch / 1000000).round().toString(),
    );
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
