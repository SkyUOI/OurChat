import 'package:fixnum/fixnum.dart';
import 'package:ourchat/google/protobuf/timestamp.pb.dart';

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
    var nanos = (datetime.microsecondsSinceEpoch % 1000000) * 1000;
    // print(datetime.microsecondsSinceEpoch);
    // print("=>timestamp$seconds,$nanos");
    timestamp = Timestamp(seconds: seconds, nanos: nanos);
  }

  void toDatetime() {
    var seconds = timestamp.seconds;
    var nanos = (timestamp.nanos / 1000).round();
    // print(timestamp);
    // print("=>datetime${seconds.toInt() * 1000000 + nanos}");
    datetime =
        DateTime.fromMicrosecondsSinceEpoch(seconds.toInt() * 1000000 + nanos);
  }

  @override
  bool operator ==(Object other) {
    if (other is OurChatTime) {
      return timestamp == other.timestamp;
    }
    return false;
  }

  @override
  int get hashCode => timestamp.hashCode;
}
