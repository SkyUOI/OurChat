import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/session/join_session/v1/join_session.pb.dart';
import 'state.dart';

/// Dialog asking the server to join a session the current account is not a
/// member of yet (issue #289). The join request may require an approval from
/// the session administrators before it takes effect.
class JoinSessionDialog extends ConsumerStatefulWidget {
  final Int64 sessionId;

  const JoinSessionDialog({super.key, required this.sessionId});

  @override
  ConsumerState<JoinSessionDialog> createState() => _JoinSessionDialogState();
}

class _JoinSessionDialogState extends ConsumerState<JoinSessionDialog> {
  final TextEditingController leaveMessageController = TextEditingController();

  @override
  void dispose() {
    leaveMessageController.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    final thisAccountId = ref.read(thisAccountIdProvider);
    final serverId = ref.read(activeServerIdProvider)!;
    var stub = ref.read(ourChatServerProvider).newStub();
    try {
      await safeRequest(
        stub.joinSession,
        JoinSessionRequest(
          sessionId: widget.sessionId,
          leaveMessage: leaveMessageController.text,
        ),
        (grpc.GrpcError e) {
          showResultMessage(
            e.code,
            e.message,
            notFoundStatus: l10n.notFound(l10n.session),
            alreadyExistsStatus: l10n.alreadyJoinedSession,
          );
        },
        rethrowError: true,
      );
      showResultMessage(okStatusCode, null, okStatus: l10n.joinRequestSent);
      await ref
          .read(ourChatAccountProvider(serverId, thisAccountId!).notifier)
          .getAccountInfo(ignoreCache: true);
      await ref.read(sessionProvider.notifier).loadSessions();
    } catch (e) {
      // Error already surfaced by safeRequest; keep the dialog open so the
      // user can retry.
      return;
    }
    if (mounted) {
      Navigator.pop(context);
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(l10n.joinSession),
      content: Form(
        child: SizedBox(
          width: 320,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Row(
                children: [
                  Padding(
                    padding: const EdgeInsets.only(right: 5.0),
                    child: Text(l10n.sessionId),
                  ),
                  SelectableText(widget.sessionId.toString()),
                ],
              ),
              TextFormField(
                controller: leaveMessageController,
                decoration: InputDecoration(
                  label: Text(l10n.joinSessionLeaveMessage),
                ),
                maxLines: 3,
              ),
            ],
          ),
        ),
      ),
      actions: [
        IconButton(
          onPressed: _submit,
          tooltip: l10n.send,
          icon: const Icon(Icons.check),
        ),
        IconButton(
          onPressed: () {
            Navigator.pop(context);
          },
          icon: const Icon(Icons.close),
        ),
      ],
    );
  }
}
