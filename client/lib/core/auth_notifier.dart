import 'package:fixnum/fixnum.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/auth/authorize/v1/authorize.pb.dart';
import 'package:ourchat/service/auth/register/v1/register.pb.dart';
import 'package:ourchat/service/auth/v1/auth.pbgrpc.dart';

part 'auth_notifier.freezed.dart';
part 'auth_notifier.g.dart';

/// 认证状态
@freezed
abstract class AuthState with _$AuthState {
  const factory AuthState({
    @Default(false) bool isLoading,
    String? error,
    Int64? accountId,
    String? token,
    String? ocid,
  }) = _AuthState;

  const AuthState._();

  bool get isAuthenticated => accountId != null && token != null;
}

@Riverpod(keepAlive: true)
class AuthNotifier extends _$AuthNotifier {
  @override
  AuthState build() {
    // 暂时不自动恢复登录状态，由上层处理
    return AuthState();
  }

  /// 登录：使用邮箱或 OCID 和密码
  Future<bool> login({
    required String password,
    String? ocid,
    String? email,
  }) async {
    if (ocid == null && email == null) {
      state = state.copyWith(error: '邮箱或 OCID 必须提供其一');
      return false;
    }

    state = state.copyWith(isLoading: true, error: null);
    try {
      final server = ref.read(ourChatServerProvider);
      final channel = server.channel;
      final authClient = AuthServiceClient(channel);

      final request = AuthRequest(password: password);
      if (email != null) {
        request.email = email;
      } else {
        request.ocid = ocid!;
      }

      final response = await authClient.auth(request);
      logger.i('登录成功，用户 ID: ${response.id}, OCID: ${response.ocid}');

      // 更新 token 到拦截器
      server.interceptor ??= OurChatInterceptor();
      server.interceptor!.setToken(response.token);

      // 更新状态
      state = state.copyWith(
        isLoading: false,
        accountId: response.id,
        token: response.token,
        ocid: response.ocid,
      );

      // 更新应用状态中的当前账户 ID
      ref.read(thisAccountIdProvider.notifier).setAccountId(response.id);

      return true;
    } on GrpcError catch (e) {
      final errorMessage = _handleAuthError(e);
      state = state.copyWith(isLoading: false, error: errorMessage);
      logger.e('登录失败: $errorMessage');
      return false;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: '登录失败: $e');
      logger.e('登录失败: $e');
      return false;
    }
  }

  /// 注册：使用邮箱、密码和用户名
  Future<bool> register({
    required String email,
    required String password,
    required String username,
    List<int>? publicKey,
  }) async {
    state = state.copyWith(isLoading: true, error: null);
    try {
      final server = ref.read(ourChatServerProvider);
      final channel = server.channel;
      final authClient = AuthServiceClient(channel);

      final request = RegisterRequest(
        email: email,
        password: password,
        name: username,
        publicKey: publicKey ?? <int>[],
      );

      final response = await authClient.register(request);
      logger.i('注册成功，用户 ID: ${response.id}, OCID: ${response.ocid}');

      // 更新 token 到拦截器
      server.interceptor ??= OurChatInterceptor();
      server.interceptor!.setToken(response.token);

      // 更新状态
      state = state.copyWith(
        isLoading: false,
        accountId: response.id,
        token: response.token,
        ocid: response.ocid,
      );

      // 更新应用状态中的当前账户 ID
      ref.read(thisAccountIdProvider.notifier).setAccountId(response.id);

      return true;
    } on GrpcError catch (e) {
      final errorMessage = _handleAuthError(e);
      state = state.copyWith(isLoading: false, error: errorMessage);
      logger.e('注册失败: $errorMessage');
      return false;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: '注册失败: $e');
      logger.e('注册失败: $e');
      return false;
    }
  }

  /// 注销
  void logout() {
    final server = ref.read(ourChatServerProvider);
    if (server.interceptor != null) {
      server.interceptor!.setToken('');
    }

    state = AuthState();

    // 清除应用状态中的当前账户 ID
    ref.read(thisAccountIdProvider.notifier).clear();

    logger.i('用户已注销');
  }

  /// 处理认证相关的 gRPC 错误
  String _handleAuthError(GrpcError e) {
    switch (e.code) {
      case 5: // NOT_FOUND
        return '用户不存在';
      case 7: // PERMISSION_DENIED
        return '密码错误';
      case 3: // INVALID_ARGUMENT
        return '请求参数无效';
      case 6: // ALREADY_EXISTS (注册时)
        return '邮箱或用户名已存在';
      default:
        return e.message ?? '认证失败，错误代码: ${e.code}';
    }
  }
}
