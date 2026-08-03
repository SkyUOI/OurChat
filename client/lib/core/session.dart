import 'dart:convert';
import 'package:drift/drift.dart' hide JsonKey;
import 'package:fixnum/fixnum.dart';
import 'package:grpc/grpc.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/database.dart' as db;
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/session/get_role/v1/get_role.pb.dart';
import 'package:ourchat/service/ourchat/session/get_session_info/v1/get_session_info.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'session.freezed.dart';
part 'session.g.dart';

@freezed
abstract class OcSessionData with _$OcSessionData {
  factory OcSessionData({
    required Int64 sessionId,
    required String name,
    required String description,
    String? avatarKey,
    required OurChatTime createdTime,
    required OurChatTime updatedTime,
    required List<Int64> members,
    required Map<Int64, Int64> roles,
    required int size,
    required List<int> myPermissions,
    String? displayName,
    required DateTime lastCheckTime,
  }) = _OcSessionData;
}

@Riverpod(keepAlive: true)
class OurChatSession extends _$OurChatSession {
  late OurChatServiceClient _stub;

  /// Server id this session provider is scoped to (family argument).
  String get _serverId => serverId;
  db.OurChatDatabase? _privateDB;
  bool _isFetching = false;

  /// The active account key if it is on this provider's server, else null.
  AccountKey? get _activeKey {
    final k = ref.read(activeAccountProvider);
    if (k == null || k.serverId != _serverId) return null;
    return k;
  }

  @override
  OcSessionData build(String serverId, Int64 sessionId) {
    _initStub();
    return OcSessionData(
      sessionId: sessionId,
      name: '',
      description: '',
      avatarKey: null,
      createdTime: OurChatTime.fromDatetime(DateTime(0)),
      updatedTime: OurChatTime.fromDatetime(DateTime(0)),
      members: [],
      roles: {},
      size: 0,
      myPermissions: [],
      displayName: null,
      lastCheckTime: DateTime(0),
    );
  }

  void _initStub() {
    final inst = instanceForServer(ref.container, _serverId);
    if (inst != null) {
      _stub = OurChatServiceClient(
        inst.server.channel,
        interceptors: inst.server.interceptor != null
            ? [inst.server.interceptor!]
            : [],
      );
      _privateDB = inst.privateDB;
    } else {
      final srv = ref.read(ourChatServerProvider);
      _stub = OurChatServiceClient(
        srv.channel,
        interceptors: srv.interceptor != null ? [srv.interceptor!] : [],
      );
      _privateDB = privateDB;
    }
  }

  void recreateStub() {
    _initStub();
  }

  String getDisplayName() {
    if (state.name.isNotEmpty) {
      return state.name;
    }
    if (state.displayName == null) {
      return l10n.newSession;
    }
    return state.displayName!;
  }

  String avatarUrl() {
    final base =
        instanceForServer(ref.container, _serverId)?.server.baseUrl() ??
        ref.read(ourChatServerProvider).baseUrl();
    return "$base/avatar?session_id=$sessionId&avatar_key=${state.avatarKey ?? ''}";
  }

  Future getSessionInfo({bool ignoreCache = false}) async {
    final sessionId = state.sessionId;
    logger.d("get session info for id: ${sessionId.toString()}");
    final activeKey = _activeKey;
    if (_isFetching) {
      while (_isFetching) {
        await Future.delayed(Duration(milliseconds: 10));
      }
      return true;
    }
    _isFetching = true;

    // Check cache freshness
    if (!ignoreCache &&
        DateTime.now().difference(state.lastCheckTime).inMinutes < 5) {
      _isFetching = false;
      logger.d("use session info cache");
      return true;
    }

    List<QueryValues> queryValues = [];
    var localSessionData =
        await (publicDB.select(publicDB.publicSession)..where(
              (u) =>
                  u.serverId.equals(_serverId) &
                  u.sessionId.equals(BigInt.from(sessionId.toInt())),
            ))
            .getSingleOrNull();
    bool publicNeedUpdate = false, privateNeedUpdate = false;
    if (localSessionData == null) {
      publicNeedUpdate = true;
    } else {
      logger.d("get session updated time");
      try {
        GetSessionInfoResponse res = await safeRequest(
          _stub.getSessionInfo,
          GetSessionInfoRequest(
            sessionId: sessionId,
            queryValues: [QueryValues.QUERY_VALUES_UPDATED_TIME],
          ),
          getSessionInfoOnError,
          rethrowError: true,
        );
        publicNeedUpdate =
            (OurChatTime.fromTimestamp(res.updatedTime) !=
            OurChatTime.fromDatetime(localSessionData.updatedTime));
      } catch (e) {
        _isFetching = false;
        return false;
      }
    }
    final isActiveMember =
        activeKey != null &&
        ref
                .read(
                  ourChatAccountProvider(
                    activeKey.serverId,
                    activeKey.accountId,
                  ),
                )
                .sessions
                .contains(sessionId) ==
            true;
    privateNeedUpdate = isActiveMember && publicNeedUpdate;
    logger.d(
      "sessionId: $sessionId, session public need update: $publicNeedUpdate, private need update: $privateNeedUpdate",
    );
    if (publicNeedUpdate) {
      logger.d("get session public info");
      queryValues.addAll([
        QueryValues.QUERY_VALUES_NAME,
        QueryValues.QUERY_VALUES_AVATAR_KEY,
        QueryValues.QUERY_VALUES_CREATED_TIME,
        QueryValues.QUERY_VALUES_UPDATED_TIME,
        QueryValues.QUERY_VALUES_SIZE,
        QueryValues.QUERY_VALUES_DESCRIPTION,
      ]);
    } else {
      state = state.copyWith(
        name: localSessionData!.name,
        avatarKey: localSessionData.avatarKey,
        createdTime: OurChatTime.fromDatetime(localSessionData.createdTime),
        updatedTime: OurChatTime.fromDatetime(localSessionData.updatedTime),
        size: localSessionData.size,
        description: localSessionData.description,
      );
    }
    var pDB = _privateDB!;
    var localSessionPrivateData =
        await (pDB.select(
              pDB.session,
            )..where((u) => u.sessionId.equals(BigInt.from(sessionId.toInt()))))
            .getSingleOrNull();
    if (localSessionPrivateData == null && isActiveMember) {
      privateNeedUpdate = true;
    }
    if (privateNeedUpdate) {
      logger.d("get session private info");
      queryValues.addAll([
        QueryValues.QUERY_VALUES_MEMBERS,
        QueryValues.QUERY_VALUES_ROLES,
      ]);
    } else if (isActiveMember) {
      // get from local db
      var localSessionPrivateData =
          await (pDB.select(pDB.session)..where(
                (u) => u.sessionId.equals(BigInt.from(sessionId.toInt())),
              ))
              .getSingle();
      var jsonMembers = jsonDecode(localSessionPrivateData.members);
      var jsonRoles = jsonDecode(localSessionPrivateData.roles);
      var jsonMyPermissions = jsonDecode(localSessionPrivateData.myPermissions);

      var myPerms = <int>[];
      for (int i = 0; i < jsonMyPermissions.length; i++) {
        myPerms.add(jsonMyPermissions[i]);
      }

      var mems = <Int64>[];
      for (int i = 0; i < jsonMembers.length; i++) {
        mems.add(Int64.parseInt(jsonMembers[i].toString()));
      }
      var rols = <Int64, Int64>{};
      jsonRoles.forEach(
        (key, value) =>
            rols[Int64.parseInt(key)] = Int64.parseInt(value.toString()),
      );

      state = state.copyWith(
        members: mems,
        roles: rols,
        myPermissions: myPerms,
      );
    }
    try {
      GetSessionInfoResponse res = await safeRequest(
        _stub.getSessionInfo,
        GetSessionInfoRequest(sessionId: sessionId, queryValues: queryValues),
        getSessionInfoOnError,
        rethrowError: true,
      );

      if (publicNeedUpdate) {
        state = state.copyWith(
          name: res.name,
          avatarKey: res.avatarKey,
          createdTime: OurChatTime.fromTimestamp(res.createdTime),
          updatedTime: OurChatTime.fromTimestamp(res.updatedTime),
          size: res.size.toInt(),
          description: res.description,
        );

        if (localSessionData == null) {
          publicDB
              .into(publicDB.publicSession)
              .insert(
                db.PublicSessionData(
                  serverId: _serverId,
                  sessionId: BigInt.from(sessionId.toInt()),
                  name: res.name,
                  createdTime: OurChatTime.fromTimestamp(
                    res.createdTime,
                  ).datetime,
                  updatedTime: OurChatTime.fromTimestamp(
                    res.updatedTime,
                  ).datetime,
                  size: res.size.toInt(),
                  description: res.description,
                ),
              );
        } else {
          (publicDB.update(publicDB.publicSession)..where(
                (u) =>
                    u.serverId.equals(_serverId) &
                    u.sessionId.equals(
                      BigInt.from(int.parse(sessionId.toString())),
                    ),
              ))
              .write(
                db.PublicSessionCompanion(
                  name: Value(res.name),
                  avatarKey: Value(res.avatarKey),
                  createdTime: Value(
                    OurChatTime.fromTimestamp(res.createdTime).datetime,
                  ),
                  updatedTime: Value(
                    OurChatTime.fromTimestamp(res.updatedTime).datetime,
                  ),
                  size: Value(res.size.toInt()),
                  description: Value(res.description),
                ),
              );
        }
      }

      if (privateNeedUpdate) {
        var mems = res.members;
        var rols = <Int64, Int64>{};
        for (int i = 0; i < res.roles.length; i++) {
          rols[res.roles[i].userId] = res.roles[i].role;
        }
        var myPerms = <int>[];
        try {
          var roleRes = await safeRequest(
            _stub.getRole,
            GetRoleRequest(roleId: rols[activeKey!.accountId]),
            (GrpcError e) {
              showResultMessage(
                e.code,
                e.message,
                notFoundStatus: l10n.notFound(l10n.role),
                permissionDeniedStatus: l10n.permissionDenied(
                  l10n.notInSession,
                ),
              );
            },
          );
          for (int i = 0; i < roleRes.permissions.length; i++) {
            myPerms.add(roleRes.permissions[i].toInt());
          }
        } catch (e) {
          // continue
        }

        var intMembers = [];
        for (int i = 0; i < mems.length; i++) {
          intMembers.add(mems[i].toInt());
        }
        var intRoles = <String, dynamic>{};
        rols.forEach((key, value) => intRoles[key.toString()] = value.toInt());
        if (localSessionPrivateData == null) {
          pDB
              .into(pDB.session)
              .insert(
                db.SessionData(
                  sessionId: BigInt.from(sessionId.toInt()),
                  members: jsonEncode(intMembers),
                  roles: jsonEncode(intRoles),
                  myPermissions: jsonEncode(myPerms),
                ),
              );
        } else {
          (pDB.update(pDB.session)..where(
                (u) => u.sessionId.equals(BigInt.from(sessionId.toInt())),
              ))
              .write(
                db.SessionCompanion(
                  members: Value(jsonEncode(intMembers)),
                  roles: Value(jsonEncode(intRoles)),
                  myPermissions: Value(jsonEncode(myPerms)),
                ),
              );
        }

        String? dn;
        if (mems.length == 2) {
          Int64 otherUserId = mems.firstWhere(
            (element) => element != activeKey!.accountId,
          );
          final otherAccount = ref.read(
            ourChatAccountProvider(_serverId, otherUserId).notifier,
          );
          await otherAccount.getAccountInfo();
          dn = otherAccount.getDisplayNameOrName();
        }

        state = state.copyWith(
          members: mems,
          roles: rols,
          myPermissions: myPerms,
          displayName: dn,
        );
      }
    } catch (e) {
      _isFetching = false;
      return false;
    }

    state = state.copyWith(lastCheckTime: DateTime.now());
    _isFetching = false;
    logger.d("save session info to cache");
  }

  void getSessionInfoOnError(GrpcError e) {}
}
