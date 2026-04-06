import 'dart:typed_data';
import 'package:fixnum/fixnum.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/event.dart';
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
    @Default({}) Map<String, Uint8List> cacheFiles,
    @Default({}) Map<String, String> cacheFilesContentType,
    @Default({}) Map<String, bool> cacheFilesSendRaw,
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
    if (thisAccountId == null) return;
    List<Int64> sessionsList = [];
    Map<Int64, UserMsg> latestMsg = {};
    final accountData = ref.read(ourChatAccountProvider(thisAccountId));
    final eventSystem = ref.read(ourChatEventSystemProvider.notifier);
    for (int i = 0; i < accountData.sessions.length; i++) {
      Int64 sessionId = accountData.sessions[i];
      core_session.OurChatSession sessionNotifier = ref.read(
        core_session.ourChatSessionProvider(sessionId).notifier,
      );
      await sessionNotifier.getSessionInfo();
      if (_disposed) return;
      List<UserMsg> record = await eventSystem.getSessionEvent(
        sessionId,
        num: 1,
      );
      if (_disposed) return;
      sessionsList.add(sessionId);
      if (record.isNotEmpty) {
        latestMsg[sessionId] = record[0];
      }
    }
    state = state.copyWith(
      sessionsList: sessionsList,
      sessionLatestMsg: latestMsg,
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
    );
  }

  void openSessionTab(Int64 sessionId, String title, {List<UserMsg>? records}) {
    state = state.copyWith(
      currentSessionId: sessionId,
      tabIndex: TabType.session,
      tabTitle: title,
      currentSessionRecords: records ?? [],
      cacheFiles: {},
      cacheFilesContentType: {},
      recordLoadCnt: 1,
    );
  }

  void clearTab() {
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
    );
  }

  void addNeedUploadFile(String path) {
    state = state.copyWith(needUploadFiles: [...state.needUploadFiles, path]);
  }

  void updateCacheFiles(
    Map<String, Uint8List> files,
    Map<String, String> contentTypes,
    Map<String, bool> sendRaw,
  ) {
    state = state.copyWith(
      cacheFiles: files,
      cacheFilesContentType: contentTypes,
      cacheFilesSendRaw: sendRaw,
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
