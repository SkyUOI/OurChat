import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:ourchat/const.dart';
import 'package:ourchat/main.dart';
import 'ourchat/ourchat_account.dart';
import 'package:provider/provider.dart';

class Auth extends StatelessWidget {
  const Auth({
    super.key,
  });
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: DefaultTabController(
          length: 2,
          child: Column(
            children: [
              TabBar(
                tabs: [
                  Tab(text: AppLocalizations.of(context)!.login),
                  Tab(text: AppLocalizations.of(context)!.register)
                ],
              ),
              const Expanded(
                  child: TabBarView(
                children: [Login(), Register()],
              )),
            ],
          )),
    );
  }
}

class Login extends StatefulWidget {
  const Login({
    super.key,
  });

  @override
  State<Login> createState() => _LoginState();
}

class _LoginState extends State<Login> {
  String account = "", password = "", errorText = "";
  bool showPassword = false;
  @override
  Widget build(BuildContext context) {
    var key = GlobalKey<FormState>();
    var ourchatAppState = context.watch<OurchatAppState>();
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
                    const Padding(
                      padding: EdgeInsets.all(10.0),
                      child: SizedBox(
                          height: 100.0, width: 100.0, child: Placeholder()),
                    ),
                    TextFormField(
                      initialValue: account,
                      decoration: InputDecoration(
                          label: Text(
                              "${AppLocalizations.of(context)!.ocid}/${AppLocalizations.of(context)!.email}")),
                      onSaved: (newValue) {
                        setState(() {
                          account = newValue!;
                        });
                      },
                    ),
                    TextFormField(
                      initialValue: password,
                      decoration: InputDecoration(
                        label: Text(AppLocalizations.of(context)!.password),
                      ),
                      onSaved: (newValue) {
                        setState(() {
                          password = newValue!;
                        });
                      },
                      obscureText: !showPassword,
                    ),
                    CheckboxListTile(
                        dense: true,
                        contentPadding: const EdgeInsets.all(0.0),
                        controlAffinity: ListTileControlAffinity.leading,
                        title: Text(AppLocalizations.of(context)!.showPassword),
                        value: showPassword,
                        onChanged: (value) {
                          setState(() {
                            key.currentState!.save();
                            showPassword = !showPassword;
                          });
                        }),
                    Padding(
                      padding: const EdgeInsets.all(10.0),
                      child: ElevatedButton(
                          onPressed: () async {
                            errorText = "";
                            key.currentState!.save();
                            OurchatAccount ocAccount =
                                OurchatAccount(ourchatAppState.server!);
                            String? email, ocid;
                            if (account.contains('@')) {
                              email = account;
                            } else {
                              ocid = account;
                            }
                            var res =
                                await ocAccount.login(password, ocid, email);
                            if (res == okStatusCode) {
                              ourchatAppState.where = homeUi;
                              ourchatAppState.thisAccount = ocAccount;
                              ourchatAppState.thisAccount!.getAccountInfo();
                              ourchatAppState.update();
                            } else {
                              setState(() {
                                switch (res) {
                                  case internalStatusCode:
                                    errorText = AppLocalizations.of(context)!
                                        .serverError;
                                    break;
                                  case unavailableStatusCode:
                                    errorText = AppLocalizations.of(context)!
                                        .serverStatusUnderMaintenance;
                                    break;
                                  case notFoundStatusCode:
                                    errorText = AppLocalizations.of(context)!
                                        .userNotFound;
                                    break;
                                  case invalidArgumentStatusCode:
                                    errorText = AppLocalizations.of(context)!
                                        .internalError;
                                    break;
                                  case unauthenticatedStatusCode:
                                    errorText = AppLocalizations.of(context)!
                                        .incorrectPassword;
                                    break;
                                  default:
                                    errorText = AppLocalizations.of(context)!
                                        .unknownError;
                                    break;
                                }
                              });
                            }
                          },
                          child: Text(AppLocalizations.of(context)!.login)),
                    ),
                    Text(errorText, style: const TextStyle(color: Colors.red))
                  ],
                )),
            Flexible(flex: 1, child: Container())
          ],
        ));
  }
}

class Register extends StatefulWidget {
  const Register({
    super.key,
  });

  @override
  State<Register> createState() => _RegisterState();
}

class _RegisterState extends State<Register> {
  String email = "", password = "", username = "", errorText = "";
  bool showPassword = false;
  @override
  Widget build(BuildContext context) {
    var key = GlobalKey<FormState>();
    var ourchatAppState = context.watch<OurchatAppState>();
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
                      initialValue: username,
                      decoration: InputDecoration(
                          label: Text(AppLocalizations.of(context)!.username)),
                      onSaved: (newValue) {
                        setState(() {
                          username = newValue!;
                        });
                      },
                    ),
                    TextFormField(
                      initialValue: email,
                      decoration: InputDecoration(
                          label: Text(AppLocalizations.of(context)!.email)),
                      onSaved: (newValue) {
                        setState(() {
                          email = newValue!;
                        });
                      },
                    ),
                    TextFormField(
                      initialValue: password,
                      decoration: InputDecoration(
                        label: Text(AppLocalizations.of(context)!.password),
                      ),
                      onSaved: (newValue) {
                        setState(() {
                          password = newValue!;
                        });
                      },
                      obscureText: !showPassword,
                    ),
                    CheckboxListTile(
                        dense: true,
                        contentPadding: const EdgeInsets.all(0.0),
                        controlAffinity: ListTileControlAffinity.leading,
                        title: Text(AppLocalizations.of(context)!.showPassword),
                        value: showPassword,
                        onChanged: (value) {
                          setState(() {
                            key.currentState!.save();
                            showPassword = !showPassword;
                          });
                        }),
                    Padding(
                      padding: const EdgeInsets.all(10.0),
                      child: ElevatedButton(
                          onPressed: () async {
                            errorText = "";
                            key.currentState!.save();
                            OurchatAccount ocAccount =
                                OurchatAccount(ourchatAppState.server!);
                            var res = await ocAccount.register(
                                password, username, email);
                            if (res == okStatusCode) {
                              ourchatAppState.where = homeUi;
                              ourchatAppState.thisAccount = ocAccount;
                              ourchatAppState.thisAccount!.getAccountInfo();
                              ourchatAppState.update();
                            } else {
                              setState(() {
                                switch (res) {
                                  case internalStatusCode:
                                    errorText = AppLocalizations.of(context)!
                                        .serverError;
                                    break;
                                  case unavailableStatusCode:
                                    errorText = AppLocalizations.of(context)!
                                        .serverStatusUnderMaintenance;
                                    break;
                                  case alreadyExistsStatusCode:
                                    errorText = AppLocalizations.of(context)!
                                        .emailExists;
                                    break;
                                  default:
                                    errorText = AppLocalizations.of(context)!
                                        .unknownError;
                                    break;
                                }
                              });
                            }
                          },
                          child: Text(AppLocalizations.of(context)!.register)),
                    ),
                    Text(errorText, style: const TextStyle(color: Colors.red))
                  ],
                )),
            Flexible(flex: 1, child: Container())
          ],
        ));
  }
}
