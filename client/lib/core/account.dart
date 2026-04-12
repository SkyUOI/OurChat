import 'dart:convert';
import 'package:drift/drift.dart' hide JsonKey;
import 'package:grpc/grpc.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/database.dart' as db;
import 'package:ourchat/service/ourchat/get_account_info/v1/get_account_info.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:fixnum/fixnum.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'account.freezed.dart';
part 'account.g.dart';

@freezed
abstract class AccountData with _$AccountData {
  factory AccountData({
    required Int64 id,
    required String username,
    required String ocid,
    String? avatarKey,
    String? displayName,
    String? status,
    String? email,
    required bool isMe,
    required OurChatTime publicUpdateTime,
    required OurChatTime updatedTime,
    required OurChatTime registerTime,
    required DateTime lastCheckTime,
    required List<Int64> friends,
    required List<Int64> sessions,
  }) = _AccountData;
}

@Riverpod(keepAlive: true)
class OurChatAccount extends _$OurChatAccount {
  @override
  AccountData build(Int64 id) {
    // 初始化 _stub
    final server = ref.read(ourChatServerProvider);
    _stub = OurChatServiceClient(
      server.channel,
      interceptors: server.interceptor != null ? [server.interceptor!] : [],
    );

    // 尝试从缓存加载账户数据
    // 这里可以查询本地数据库，或返回默认值
    // 暂时返回一个默认的 AccountStateData，稍后通过 getAccountInfo 填充
    return AccountData(
      id: id,
      username: '',
      ocid: '',
      avatarKey: null,
      displayName: null,
      status: null,
      email: null,
      isMe: false,
      publicUpdateTime: OurChatTime.fromTimestamp(Timestamp()),
      updatedTime: OurChatTime.fromTimestamp(Timestamp()),
      registerTime: OurChatTime.fromTimestamp(Timestamp()),
      lastCheckTime: DateTime(0),
      friends: [],
      sessions: [],
    );
  }

  late OurChatServiceClient _stub;
  // 客户端独有字段，仅isMe为True时使用
  OurChatTime _latestMsgTime = OurChatTime.fromTimestamp(Timestamp());
  bool _isFetching = false;

  void recreateStub() {
    final server = ref.read(ourChatServerProvider);
    _stub = OurChatServiceClient(
      server.channel,
      interceptors: [server.interceptor!],
    );
  }

  OurChatTime getLatestMsgTime() => _latestMsgTime;
  void setLatestMsgTime(OurChatTime time) {
    _latestMsgTime = time;
  }

  Future<bool> getAccountInfo({bool ignoreCache = false}) async {
    final id = state.id; // 当前账户ID
    final thisAccountId = ref.read(thisAccountIdProvider);
    logger.d("get account info for id: ${id.toString()}");

    // 防止重复请求
    if (_isFetching) {
      while (_isFetching) {
        await Future.delayed(Duration(milliseconds: 10));
      }
      return true;
    }
    _isFetching = true;

    // 检查缓存 freshness
    if (!ignoreCache &&
        DateTime.now().difference(state.lastCheckTime).inMinutes < 5) {
      _isFetching = false;
      logger.d("use account info cache");
      return true;
    }

    List<QueryValues> requestValues = [];
    db.OurChatDatabase pdb = privateDB!;
    bool isMe = state.isMe;
    if (thisAccountId != null && thisAccountId == id) {
      isMe = true;
      state = (state.copyWith(isMe: true));
    }
    bool publicDataNeedUpdate = false, privateDataNeedUpdate = false;
    var publicData = await (publicDB.select(
      publicDB.publicAccount,
    )..where((u) => u.id.equals(BigInt.from((id.toInt()))))).getSingleOrNull();
    var privateData = await (pdb.select(
      pdb.account,
    )..where((u) => u.id.equals(BigInt.from((id.toInt()))))).getSingleOrNull();
    if (publicData == null) {
      publicDataNeedUpdate = true;
    } else {
      logger.d("get account public updated time");
      try {
        GetAccountInfoResponse res = await safeRequest(
          _stub.getAccountInfo,
          GetAccountInfoRequest(
            id: id,
            requestValues: [QueryValues.QUERY_VALUES_PUBLIC_UPDATED_TIME],
          ),
          getAccountInfoOnError,
          rethrowError: true,
        );
        if (OurChatTime.fromTimestamp(res.publicUpdatedTime) !=
            OurChatTime.fromDatetime(publicData.publicUpdateTime)) {
          publicDataNeedUpdate = true;
        }
      } catch (e) {
        _isFetching = false;
        return false;
      }
    }
    if (!publicDataNeedUpdate) {
      // 使用本地缓存
      state = (state.copyWith(
        username: publicData!.username,
        ocid: publicData.ocid,
        avatarKey: publicData.avatarKey,
        status: publicData.status,
        publicUpdateTime: OurChatTime.fromDatetime(publicData.publicUpdateTime),
      ));
    }
    if (privateData == null) {
      if (isMe) {
        privateDataNeedUpdate = true;
      }
    } else {
      logger.d("get account private updated time");
      try {
        GetAccountInfoResponse res = await safeRequest(
          _stub.getAccountInfo,
          GetAccountInfoRequest(
            id: id,
            requestValues: [QueryValues.QUERY_VALUES_UPDATED_TIME],
          ),
          getAccountInfoOnError,
          rethrowError: true,
        );
        if (OurChatTime.fromTimestamp(res.updatedTime) !=
            OurChatTime.fromDatetime(privateData.updateTime)) {
          privateDataNeedUpdate = true;
        }
      } catch (e) {
        _isFetching = false;
        return false;
      }
    }
    logger.d(
      "accountId: $id,isMe: $isMe, private data need update: $privateDataNeedUpdate, public data need update: $publicDataNeedUpdate",
    );
    if (!privateDataNeedUpdate && isMe) {
      // 使用本地缓存
      state = (state.copyWith(
        updatedTime: OurChatTime.fromDatetime(privateData!.updateTime),
        email: privateData.email,
        friends: [],
        sessions: [],
        registerTime: OurChatTime.fromDatetime(privateData.registerTime),
      ));
      // 解析friends和sessions
      List<Int64> friends = [];
      List<dynamic> friendsList = jsonDecode(privateData.friendsJson);
      for (int i = 0; i < friendsList.length; i++) {
        friends.add(Int64.parseInt(friendsList[i].toString()));
      }
      List<Int64> sessions = [];
      List<dynamic> sessionsList = jsonDecode(privateData.sessionsJson);
      for (int i = 0; i < sessionsList.length; i++) {
        sessions.add(Int64.parseInt(sessionsList[i].toString()));
      }
      state = (state.copyWith(friends: friends, sessions: sessions));
    }

    if (publicDataNeedUpdate) {
      requestValues.addAll([
        QueryValues.QUERY_VALUES_AVATAR_KEY,
        QueryValues.QUERY_VALUES_USER_NAME,
        QueryValues.QUERY_VALUES_PUBLIC_UPDATED_TIME,
        QueryValues.QUERY_VALUES_STATUS,
        QueryValues.QUERY_VALUES_OCID,
      ]);
    }
    if (privateDataNeedUpdate) {
      requestValues.addAll([
        QueryValues.QUERY_VALUES_UPDATED_TIME,
        QueryValues.QUERY_VALUES_SESSIONS,
        QueryValues.QUERY_VALUES_FRIENDS,
        QueryValues.QUERY_VALUES_EMAIL,
        QueryValues.QUERY_VALUES_REGISTER_TIME,
      ]);
    }
    if (publicDataNeedUpdate ||
        privateDataNeedUpdate ||
        (thisAccountId != null &&
            thisAccountId != id &&
            ref
                    .read(ourChatAccountProvider(thisAccountId))
                    .friends
                    .contains(id) ==
                true)) {
      logger.d("get account info");
      try {
        GetAccountInfoResponse res = await safeRequest(
          _stub.getAccountInfo,
          GetAccountInfoRequest(id: id, requestValues: requestValues),
          getAccountInfoOnError,
          rethrowError: true,
        );
        if (publicDataNeedUpdate) {
          await updatePublicData(res, publicData != null);
        }
        if (privateDataNeedUpdate) {
          await updatePrivateData(res, privateData != null);
        }
        if (thisAccountId != null &&
            ref
                    .read(ourChatAccountProvider(thisAccountId))
                    .friends
                    .contains(id) ==
                true) {
          // get displayname
          logger.d("get account display_name info");
          res = await safeRequest(
            _stub.getAccountInfo,
            GetAccountInfoRequest(
              id: id,
              requestValues: [QueryValues.QUERY_VALUES_DISPLAY_NAME],
            ),
            getAccountInfoOnError,
            rethrowError: true,
          );
          state = (state.copyWith(displayName: res.displayName));
        }
      } catch (e) {
        _isFetching = false;
        return false;
      }
    }
    state = (state.copyWith(lastCheckTime: DateTime.now()));
    _isFetching = false;
    logger.d("save account info to cache");
    return true;
  }

  Future updatePublicData(GetAccountInfoResponse res, bool isDataExist) async {
    logger.d("get account public info");
    state = (state.copyWith(
      avatarKey: res.avatarKey,
      username: res.userName,
      publicUpdateTime: OurChatTime.fromTimestamp(res.publicUpdatedTime),
      status: res.status,
      ocid: res.ocid,
    ));
    final id = state.id;
    if (isDataExist) {
      // 更新数据
      (publicDB.update(
        publicDB.publicAccount,
      )..where((u) => u.id.equals(BigInt.from(id.toInt())))).write(
        db.PublicAccountCompanion(
          avatarKey: Value(res.avatarKey),
          username: Value(res.userName),
          publicUpdateTime: Value(
            OurChatTime.fromTimestamp(res.publicUpdatedTime).datetime,
          ),
          status: Value(res.status),
          ocid: Value(res.ocid),
        ),
      );
    } else {
      publicDB
          .into(publicDB.publicAccount)
          .insert(
            db.PublicAccountData(
              id: BigInt.from(id.toInt()),
              avatarKey: res.avatarKey,
              username: res.userName,
              publicUpdateTime: OurChatTime.fromTimestamp(
                res.publicUpdatedTime,
              ).datetime,
              status: res.status,
              ocid: res.ocid,
            ),
          );
    }
  }

  Future updatePrivateData(GetAccountInfoResponse res, bool isDataExist) async {
    logger.d("update account private info");
    state = (state.copyWith(
      updatedTime: OurChatTime.fromTimestamp(res.updatedTime),
      email: res.email,
      friends: res.friends,
      sessions: res.sessions,
      registerTime: OurChatTime.fromTimestamp(res.registerTime),
    ));
    final id = state.id;
    var intFriendsId = [];
    var intSessionsId = [];
    for (int i = 0; i < res.friends.length; i++) {
      intFriendsId.add(res.friends[i].toInt());
    }
    for (int i = 0; i < res.sessions.length; i++) {
      intSessionsId.add(res.sessions[i].toInt());
    }
    var localDb = privateDB!;
    if (isDataExist) {
      (localDb.update(
        localDb.account,
      )..where((u) => u.id.equals(BigInt.from(id.toInt())))).write(
        db.AccountCompanion(
          email: Value(res.email),
          registerTime: Value(
            OurChatTime.fromTimestamp(res.registerTime).datetime,
          ),
          updateTime: Value(
            OurChatTime.fromTimestamp(res.updatedTime).datetime,
          ),
          friendsJson: Value(jsonEncode(intFriendsId)),
          sessionsJson: Value(jsonEncode(intSessionsId)),
          latestMsgTime: Value(_latestMsgTime.datetime),
        ),
      );
    } else {
      localDb
          .into(localDb.account)
          .insert(
            db.AccountData(
              id: BigInt.from(id.toInt()),
              email: res.email,
              registerTime: OurChatTime.fromTimestamp(
                res.registerTime,
              ).datetime,
              updateTime: OurChatTime.fromTimestamp(res.updatedTime).datetime,
              friendsJson: jsonEncode(intFriendsId),
              sessionsJson: jsonEncode(intSessionsId),
              latestMsgTime: _latestMsgTime.datetime,
            ),
          );
    }
  }

  void updateLatestMsgTime() {
    var pdb = privateDB!;
    final id = state.id;
    (pdb.update(
      pdb.account,
    )..where((u) => u.id.equals(BigInt.from(id.toInt())))).write(
      db.AccountCompanion(latestMsgTime: Value(_latestMsgTime.datetime)),
    );
  }

  String avatarUrl() {
    return "${ref.read(ourChatServerProvider).baseUrl()}/avatar?${state.id}&avatar_key=${state.avatarKey ?? ''}";
  }

  String getNameWithDisplayName() {
    final username = state.username;
    final displayName = state.displayName;
    if (displayName != null && displayName != "" && displayName != username) {
      return "$displayName ($username)";
    }
    return username;
  }

  String getDisplayNameOrName() {
    final displayName = state.displayName;
    final username = state.username;
    if (displayName != null && displayName != "") {
      return displayName;
    }
    return username;
  }

  void getAccountInfoOnError(GrpcError e) {
    showResultMessage(
      e.code,
      e.message,
      notFoundStatus: l10n.notFound(l10n.user),
      permissionDeniedStatus: l10n.permissionDenied("Get Account Info"),
      invalidArgumentStatus: l10n.internalError,
    );
  }
}
