import 'dart:typed_data';
import 'package:fixnum/fixnum.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/core/session.dart' as core_session;
import 'package:ourchat/main.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'state.freezed.dart';
part 'state.g.dart';

enum TabType { empty, session, user }

@freezed
abstract class SessionState with _$SessionState {
  factory SessionState({
    @Default(TabType.empty) TabType tabIndex,
    Int64? currentSessionId,
    Int64? currentUserId,
    @Default("") String tabTitle,
    @Default([]) List<UserMsg> currentSessionRecords,
    @Default([]) List<Int64> sessionsList,
    @Default({}) Map<Int64, UserMsg> sessionLatestMsg,
    @Default({}) Map<Int64, String> sessionServerIds,
    @Default({}) Map<String, Uint8List> cacheFiles,
    @Default({}) Map<String, String> cacheFilesContentType,
    @Default({}) Map<String, bool> cacheFilesSendRaw,
    @Default({}) Map<String, String> cacheFileNames,
    @Default([]) List<String> needUploadFiles,
    @Default(1) int recordLoadCnt,
    @Default(0) double lastPixels,
    @Default(false) bool sessionsLoading,
  }) = _SessionState;
}

@riverpod
class InputText extends _$InputText {
  @override
  String build() {
    return "";
  }

  void setText(String text) {
    state = text;
  }
}

@riverpod
class QuoteTarget extends _$QuoteTarget {
  @override
  UserMsg? build() {
    return null;
  }

  void setQuote(UserMsg msg) {
    state = msg;
  }

  void clear() {
    state = null;
  }
}

@riverpod
class SessionNotifier extends _$SessionNotifier {
  bool _disposed = false;

  @override
  SessionState build() {
    _disposed = false;
    ref.onDispose(() => _disposed = true);
    return SessionState();
  }

  void receiveMsg(UserMsg eventObj) {
    final latestMsg = Map<Int64, UserMsg>.from(state.sessionLatestMsg);
    latestMsg[eventObj.sessionId!] = eventObj;
    if (state.currentSessionId == eventObj.sessionId) {
      state = state.copyWith(
        sessionLatestMsg: latestMsg,
        currentSessionRecords: [eventObj, ...state.currentSessionRecords],
      );
    } else {
      state = state.copyWith(sessionLatestMsg: latestMsg);
    }
  }

  Future<void> loadSessions() async {
    state = state.copyWith(sessionsLoading: true);
    final thisAccountId = ref.read(thisAccountIdProvider);
    final serverId = ref.read(activeServerIdProvider);
    if (thisAccountId == null || serverId == null) {
      state = state.copyWith(sessionsLoading: false);
      return;
    }
    final mode = ref.read(configProvider).displayMode;
    final sessionsList = <Int64>[];
    final latestMsg = <Int64, UserMsg>{};
    final sessionServerIds = <Int64, String>{};

    Future<void> loadFromAccount(String sid, Int64 accountId) async {
      final accountData = ref.read(ourChatAccountProvider(sid, accountId));
      final eventSystem = ref.read(
        ourChatEventSystemProvider(sid, accountId).notifier,
      );
      for (int i = 0; i < accountData.sessions.length; i++) {
        Int64 sessionId = accountData.sessions[i];
        core_session.OurChatSession sessionNotifier = ref.read(
          core_session.ourChatSessionProvider(sid, sessionId).notifier,
        );
        await sessionNotifier.getSessionInfo();
        if (_disposed) return;
        List<UserMsg> record = await eventSystem.getSessionEvent(
          sessionId,
          num: 1,
        );
        if (_disposed) return;
        sessionsList.add(sessionId);
        sessionServerIds[sessionId] = sid;
        if (record.isNotEmpty) {
          latestMsg[sessionId] = record[0];
        }
      }
    }

    if (mode == UiDisplayMode.unifiedInbox) {
      // Aggregate conversations from every logged-in server/account.
      final instances = ref.read(instancesProvider);
      for (final inst in instances.values) {
        await loadFromAccount(inst.serverId, inst.accountId);
        if (_disposed) return;
      }
    } else {
      await loadFromAccount(serverId, thisAccountId);
      if (_disposed) return;
    }

    state = state.copyWith(
      sessionsList: sessionsList,
      sessionLatestMsg: latestMsg,
      sessionServerIds: sessionServerIds,
      sessionsLoading: false,
    );
  }

  void openUserTab(Int64 userId, String title) {
    state = state.copyWith(
      currentUserId: userId,
      tabIndex: TabType.user,
      tabTitle: title,
      cacheFiles: {},
      cacheFilesContentType: {},
      cacheFileNames: {},
    );
  }

  void openSessionTab(Int64 sessionId, String title, {List<UserMsg>? records}) {
    ref.read(quoteTargetProvider.notifier).clear();
    state = state.copyWith(
      currentSessionId: sessionId,
      tabIndex: TabType.session,
      tabTitle: title,
      currentSessionRecords: records ?? [],
      cacheFiles: {},
      cacheFilesContentType: {},
      cacheFileNames: {},
      recordLoadCnt: 1,
    );
  }

  void clearTab() {
    ref.read(quoteTargetProvider.notifier).clear();
    state = state.copyWith(
      tabTitle: "",
      currentUserId: null,
      currentSessionId: null,
      currentSessionRecords: [],
    );
  }

  void addRecords(List<UserMsg> records) {
    state = state.copyWith(
      currentSessionRecords: [...records, ...state.currentSessionRecords],
      recordLoadCnt: state.recordLoadCnt + 1,
    );
  }

  void setLastPixels(double pixels) {
    state = state.copyWith(lastPixels: pixels);
  }

  void updateTabTitle(String title) {
    state = state.copyWith(tabTitle: title);
  }

  void resetInputArea() {
    state = state.copyWith(
      needUploadFiles: [],
      cacheFiles: {},
      cacheFilesContentType: {},
      cacheFileNames: {},
    );
  }

  void addNeedUploadFile(String path) {
    state = state.copyWith(needUploadFiles: [...state.needUploadFiles, path]);
  }

  void updateCacheFiles(
    Map<String, Uint8List> files,
    Map<String, String> contentTypes,
    Map<String, bool> sendRaw, {
    Map<String, String>? fileNames,
  }) {
    state = state.copyWith(
      cacheFiles: files,
      cacheFilesContentType: contentTypes,
      cacheFilesSendRaw: sendRaw,
      cacheFileNames: fileNames ?? state.cacheFileNames,
    );
  }

  void clearNeedUploadFiles() {
    state = state.copyWith(needUploadFiles: []);
  }

  void switchSendRaw(String uri) {
    Map<String, bool> sendRaw = Map.from(state.cacheFilesSendRaw);
    sendRaw[uri] = !sendRaw[uri]!;
    state = state.copyWith(cacheFilesSendRaw: sendRaw);
  }
}
