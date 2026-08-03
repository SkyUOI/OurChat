import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fixnum/fixnum.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/core/secret_store.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/core/database.dart';
import 'package:ourchat/server_setting.dart';
import 'core/account.dart';
import 'core/auth_notifier.dart';
import 'core/crypto.dart';

/// Post-login/register wiring: open the private DB, build a runtime
/// [OurChatInstance], register it as active, persist the [SavedAccount], and
/// kick off the event system + account info fetch.
Future<void> _handleAuthSuccess({
  required WidgetRef ref,
  required BuildContext context,
  required Int64 accountId,
  required String accountIdent,
  required String password,
  required bool savePassword,
  String? ocid,
  String? email,
}) async {
  final authState = ref.read(authProvider);
  final serverId = authState.serverId!;
  final accountIdInt = accountId.toInt();

  // Persist the freshly generated E2EE private key for this account.
  await ref.read(authProvider.notifier).persistPrivateKey(accountId);

  // Persist the password via platform-backed secure storage (never via
  // SharedPreferences). Only when the user opted in.
  if (savePassword && password.isNotEmpty) {
    await SecretStore.saveCredential(serverId, accountIdent, password);
  }

  // Build the runtime instance for this login and register it as active.
  final server = ref.read(ourChatServerProvider);
  final newPrivateDB = OurChatDatabase(serverId, accountId);
  final instance = OurChatInstance(
    serverId: serverId,
    accountId: accountId,
    server: server,
    privateDB: newPrivateDB,
  );
  ref.read(instancesProvider.notifier).add(instance);
  ref.read(activeAccountProvider.notifier).set(instance.key);
  privateDB = newPrivateDB;

  final ocidToSave = ocid ?? authState.ocid;
  ref
      .read(configProvider.notifier)
      .upsertSavedAccount(
        SavedAccount(
          serverId: serverId,
          accountId: accountIdInt,
          ocid: ocidToSave,
          email: email,
          avatarKey: null,
          lastLoginAt: DateTime.now(),
          autoLogin: savePassword,
        ),
      );
  ref.read(configProvider.notifier).setActiveAccount(serverId, accountIdInt);

  await ref
      .read(ourChatAccountProvider(serverId, accountId).notifier)
      .getAccountInfo();
  ref
      .read(ourChatEventSystemProvider(serverId, accountId).notifier)
      .listenEvents();

  if (context.mounted) {
    Navigator.pop(context);
  }
}

/// Resolve the saved account to pre-fill on the login screen (the active one,
/// else the first saved account on the active server, else null).
SavedAccount? _savedAccountForPrefill(OurChatConfig cfg) {
  if (cfg.savedAccounts.isEmpty) return null;
  for (final a in cfg.savedAccounts) {
    if (a.serverId == cfg.activeServerId &&
        a.accountId == cfg.activeAccountId) {
      return a;
    }
  }
  return cfg.savedAccounts.first;
}

// Auth screen
class Auth extends ConsumerWidget {
  const Auth({super.key});
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final server = ref.watch(ourChatServerProvider);
    final serverName = server.serverName ?? '${server.host}:${server.port}';
    return Scaffold(
      body: SafeArea(
        child: DefaultTabController(
          length: 2,
          child: Column(
            children: [
              // Currently logged-in server + switch entry
              Padding(
                padding: const EdgeInsets.only(top: 8.0),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    const Icon(Icons.dns, size: 18),
                    const SizedBox(width: 6),
                    Flexible(
                      child: Text(serverName, overflow: TextOverflow.ellipsis),
                    ),
                    TextButton.icon(
                      style: AppStyles.defaultButtonStyle,
                      icon: const Icon(Icons.swap_horiz, size: 18),
                      label: Text(l10n.selectServer),
                      onPressed: () {
                        Navigator.pushReplacement(
                          context,
                          MaterialPageRoute(
                            builder: (context) => ServerSetting(),
                          ),
                        );
                      },
                    ),
                  ],
                ),
              ),
              TabBar(
                tabs: [
                  Tab(text: l10n.login),
                  Tab(text: l10n.register),
                ],
              ),
              const Expanded(
                child: TabBarView(children: [Login(), Register()]),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class Login extends ConsumerStatefulWidget {
  const Login({super.key});

  @override
  ConsumerState<Login> createState() => _LoginState();
}

class _LoginState extends ConsumerState<Login> {
  String account = "", password = "";
  bool savePassword = false, inited = false;

  @override
  void initState() {
    super.initState();
    _loadSavedCredential();
  }

  Future<void> _loadSavedCredential() async {
    final config = ref.read(configProvider);
    final saved = _savedAccountForPrefill(config);
    if (saved == null) return;
    final ident = saved.email ?? saved.ocid;
    if (ident == null) return;
    final savedPassword = await SecretStore.readCredential(
      saved.serverId,
      ident,
    );
    if (!mounted) return;
    setState(() {
      if (savedPassword != null && savedPassword.isNotEmpty) {
        password = savedPassword;
        savePassword = true;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    var key = GlobalKey<FormState>();
    final config = ref.read(configProvider);
    if (!inited) {
      final saved = _savedAccountForPrefill(config);
      if (saved != null) {
        account = saved.email ?? saved.ocid ?? "";
      }
      inited = true;
    }
    return SafeArea(
      child: Form(
        key: key,
        child: Row(
          children: [
            Flexible(flex: 1, child: Container()),
            Flexible(
              flex: 3,
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Padding(
                    padding: EdgeInsets.all(10.0),
                    child: SizedBox(
                      height: 100.0,
                      width: 100.0,
                      child: Image.asset("assets/images/logo.png"),
                    ),
                  ),
                  TextFormField(
                    // Account input field
                    initialValue: account,
                    decoration: InputDecoration(
                      label: Text("${l10n.ocid}/${l10n.email}"),
                    ),
                    onSaved: (newValue) {
                      setState(() {
                        account = newValue!;
                      });
                    },
                  ),
                  TextFormField(
                    // Password input field
                    initialValue: password,
                    decoration: InputDecoration(label: Text(l10n.password)),
                    onSaved: (newValue) {
                      setState(() {
                        password = newValue!;
                      });
                    },
                    obscureText: true,
                  ),
                  CheckboxListTile(
                    // Save password checkbox
                    dense: true,
                    contentPadding: const EdgeInsets.all(0.0),
                    controlAffinity: ListTileControlAffinity.leading,
                    title: Text(l10n.savePassword),
                    value: savePassword,
                    onChanged: (value) {
                      setState(() {
                        key.currentState!.save();
                        savePassword = !savePassword;
                      });
                    },
                  ),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Padding(
                        padding: EdgeInsets.all(AppStyles.mediumPadding),
                        child: ElevatedButton.icon(
                          style: AppStyles.defaultButtonStyle,
                          icon: Icon(Icons.login),
                          onPressed: () async {
                            key.currentState!.save(); // Save form
                            String? email, ocid;
                            if (account.contains('@')) {
                              // Determine email/ocid login
                              email = account;
                            } else {
                              ocid = account;
                            }
                            bool res = await ref
                                .read(authProvider.notifier)
                                .login(
                                  password: password,
                                  ocid: ocid,
                                  email: email,
                                );

                            if (res) {
                              final accountId = ref
                                  .read(authProvider)
                                  .accountId!;
                              if (context.mounted) {
                                await _handleAuthSuccess(
                                  ref: ref,
                                  context: context,
                                  accountId: accountId,
                                  accountIdent: account,
                                  password: password,
                                  savePassword: savePassword,
                                  ocid: ocid,
                                  email: email,
                                );
                              }
                            }
                          },
                          label: Text(l10n.login),
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
            Flexible(flex: 1, child: Container()),
          ],
        ),
      ),
    );
  }
}

// Register
class Register extends ConsumerStatefulWidget {
  const Register({super.key});

  @override
  ConsumerState<Register> createState() => _RegisterState();
}

class _RegisterState extends ConsumerState<Register> {
  String email = "", password = "", username = "";
  bool showPassword = false, savePassword = false;
  @override
  Widget build(BuildContext context) {
    var key = GlobalKey<FormState>();
    return Form(
      key: key,
      child: Row(
        children: [
          Flexible(flex: 1, child: Container()),
          Flexible(
            flex: 3,
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                TextFormField(
                  // Username input field
                  initialValue: username,
                  decoration: InputDecoration(label: Text(l10n.username)),
                  onSaved: (newValue) {
                    setState(() {
                      username = newValue!;
                    });
                  },
                ),
                TextFormField(
                  // Email input field
                  initialValue: email,
                  decoration: InputDecoration(label: Text(l10n.email)),
                  onSaved: (newValue) {
                    setState(() {
                      email = newValue!;
                    });
                  },
                ),
                TextFormField(
                  // Password input field
                  initialValue: password,
                  decoration: InputDecoration(label: Text(l10n.password)),
                  onSaved: (newValue) {
                    setState(() {
                      password = newValue!;
                    });
                  },
                  obscureText: !showPassword,
                ),
                CheckboxListTile(
                  // Show password checkbox
                  dense: true,
                  controlAffinity: ListTileControlAffinity.leading,
                  title: Text(l10n.show(l10n.password)),
                  value: showPassword,
                  onChanged: (value) {
                    setState(() {
                      key.currentState!.save();
                      showPassword = !showPassword;
                    });
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  controlAffinity: ListTileControlAffinity.leading,
                  title: Text(l10n.savePassword),
                  value: savePassword,
                  onChanged: (value) {
                    setState(() {
                      key.currentState!.save();
                      savePassword = !savePassword;
                    });
                  },
                ),
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Padding(
                      padding: EdgeInsets.all(AppStyles.mediumPadding),
                      child: ElevatedButton.icon(
                        style: AppStyles.defaultButtonStyle,
                        icon: Icon(Icons.app_registration),
                        onPressed: () async {
                          key.currentState!.save(); // Save form
                          // Generate RSA key pair client-side before registering.
                          // The public key goes to the server; the private key
                          // is handed to AuthNotifier for secure persistence.
                          final keyPair = generateRsaKeyPair();
                          ref
                              .read(authProvider.notifier)
                              .setPendingPrivateKey(keyPair.privateKey);
                          bool res = await ref
                              .read(authProvider.notifier)
                              .register(
                                email: email,
                                password: password,
                                username: username,
                                publicKey: keyPair.publicKey,
                              );
                          if (res) {
                            final accountId = ref.read(authProvider).accountId!;
                            if (context.mounted) {
                              await _handleAuthSuccess(
                                ref: ref,
                                context: context,
                                accountId: accountId,
                                accountIdent: email,
                                password: password,
                                savePassword: savePassword,
                                email: email,
                              );
                            }
                          }
                        },
                        label: Text(l10n.register),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          Flexible(flex: 1, child: Container()),
        ],
      ),
    );
  }
}
