import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';

/// Full-page list of server announcements, fetched via `fetchMsgs` with
/// `announcement_only = true` (newest first). The stream may switch to live
/// mode after replaying the history, so entries are appended as they arrive.
class AnnouncementListPage extends ConsumerStatefulWidget {
  const AnnouncementListPage({super.key});

  @override
  ConsumerState<AnnouncementListPage> createState() =>
      _AnnouncementListPageState();
}

class _AnnouncementListPageState extends ConsumerState<AnnouncementListPage> {
  final List<AnnouncementResponseEvent> _announcements = [];
  bool _loading = true;
  bool _fetchStarted = false;
  grpc.ResponseStream<FetchMsgsResponse>? _stream;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _fetchAnnouncements());
  }

  @override
  void dispose() {
    _stream?.cancel();
    super.dispose();
  }

  Future<void> _fetchAnnouncements() async {
    if (_fetchStarted) return;
    _fetchStarted = true;
    try {
      final stub = ref.read(ourChatServerProvider).newStub();
      _stream = stub.fetchMsgs(
        // Epoch time => replay the whole announcement history.
        FetchMsgsRequest(announcementOnly: true, time: Timestamp()),
      );
      await for (final response in _stream!) {
        if (response.whichRespondEventType() !=
            FetchMsgsResponse_RespondEventType.announcementResponse) {
          continue;
        }
        final event = announcementEventFromResponse(response);
        if (!mounted) return;
        setState(() {
          // History arrives oldest-first; show newest on top.
          _announcements.insert(0, event);
          _loading = false;
        });
      }
    } on grpc.GrpcError catch (e) {
      logger.w("failed to fetch announcements: ${e.message}");
      showResultMessage(e.code, e.message, internalStatus: l10n.serverError);
    } catch (e) {
      logger.w("failed to fetch announcements: $e");
    }
    if (mounted) {
      setState(() => _loading = false);
    }
  }

  String _formatTime(DateTime time) {
    final local = time.toLocal();
    return '${local.year}-${local.month.toString().padLeft(2, '0')}-${local.day.toString().padLeft(2, '0')} '
        '${local.hour.toString().padLeft(2, '0')}:${local.minute.toString().padLeft(2, '0')}';
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        leading: BackButton(onPressed: () => Navigator.pop(context)),
        title: Text(l10n.announcement),
      ),
      body: _loading && _announcements.isEmpty
          ? const Center(child: CircularProgressIndicator())
          : _announcements.isEmpty
          ? Center(child: Text(l10n.noAnnouncement))
          : ListView.builder(
              padding: EdgeInsets.all(AppStyles.mediumPadding),
              itemCount: _announcements.length,
              itemBuilder: (context, index) {
                final announcement = _announcements[index];
                return Card(
                  margin: EdgeInsets.symmetric(
                    vertical: AppStyles.smallPadding,
                  ),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(
                      AppStyles.defaultBorderRadius,
                    ),
                  ),
                  child: Padding(
                    padding: EdgeInsets.all(AppStyles.mediumPadding),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        if (announcement.title?.isNotEmpty ?? false)
                          Padding(
                            padding: const EdgeInsets.only(bottom: 6.0),
                            child: Text(
                              announcement.title!,
                              style: const TextStyle(
                                fontWeight: FontWeight.bold,
                                fontSize: 16,
                              ),
                            ),
                          ),
                        SelectableText(announcement.content ?? ''),
                        const SizedBox(height: 6),
                        Text(
                          '${l10n.announcementPublisherId(announcement.publisherId.toString())} · ${_formatTime(announcement.sendTime!.datetime)}',
                          style: const TextStyle(
                            color: Colors.grey,
                            fontSize: 12,
                          ),
                        ),
                      ],
                    ),
                  ),
                );
              },
            ),
    );
  }
}
