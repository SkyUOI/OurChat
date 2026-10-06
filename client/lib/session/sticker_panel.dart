import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/sticker/v1/sticker.pb.dart';

/// The "stickers" tab of the input panel (issue #147): the user's private
/// sticker collection stored on the server. Tapping a sticker sends it as an
/// image message referencing the already-uploaded file; long-pressing offers
/// to remove it from the collection.
class StickerPanel extends ConsumerStatefulWidget {
  const StickerPanel({super.key, required this.onStickerSelected});

  /// Called with the server-side file key of the tapped sticker.
  final ValueChanged<String> onStickerSelected;

  @override
  ConsumerState<StickerPanel> createState() => _StickerPanelState();
}

class _StickerPanelState extends ConsumerState<StickerPanel> {
  List<Sticker>? _stickers;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() => _loading = true);
    try {
      var stub = ref.read(ourChatServerProvider).newStub();
      final res = await safeRequest(stub.getStickers, GetStickersRequest(), (
        grpc.GrpcError e,
      ) {
        showResultMessage(e.code, e.message);
      });
      if (!mounted) return;
      setState(() {
        _stickers = res?.stickers.toList() ?? [];
        _loading = false;
      });
    } catch (e) {
      logger.w("failed to load stickers: $e");
      if (!mounted) return;
      setState(() {
        _stickers = [];
        _loading = false;
      });
    }
  }

  Future<void> _remove(Sticker sticker) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(l10n.removeSticker),
        content: Text(l10n.removeStickerConfirm),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: Text(l10n.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: Text(l10n.ok),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    var stub = ref.read(ourChatServerProvider).newStub();
    await safeRequest(
      stub.removeSticker,
      RemoveStickerRequest(fileKey: sticker.fileKey),
      (grpc.GrpcError e) {
        showResultMessage(e.code, e.message);
      },
    );
    await _load();
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }
    final stickers = _stickers ?? const <Sticker>[];
    if (stickers.isEmpty) {
      return Center(child: Text(l10n.stickersEmpty));
    }
    return GridView.builder(
      padding: const EdgeInsets.all(4.0),
      gridDelegate: SliverGridDelegateWithMaxCrossAxisExtent(
        maxCrossAxisExtent: 72,
        mainAxisSpacing: 4,
        crossAxisSpacing: 4,
      ),
      itemCount: stickers.length,
      itemBuilder: (context, index) {
        final sticker = stickers[index];
        return InkWell(
          borderRadius: BorderRadius.circular(6),
          onTap: () => widget.onStickerSelected(sticker.fileKey),
          onLongPress: () => _remove(sticker),
          child: _StickerImage(fileKey: sticker.fileKey),
        );
      },
    );
  }
}

/// One sticker cell: downloads the image through the authenticated file API
/// (cached by the cache manager) and shows it.
class _StickerImage extends ConsumerWidget {
  const _StickerImage({required this.fileKey});

  final String fileKey;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return FutureBuilder(
      future: getOurChatFile(ref, fileKey),
      builder: (context, snapshot) {
        if (snapshot.hasData) {
          return Image.memory(snapshot.data!.bytes, fit: BoxFit.contain);
        }
        if (snapshot.hasError) {
          return const Center(child: Icon(Icons.broken_image, size: 24));
        }
        return const Center(
          child: SizedBox(
            width: 16,
            height: 16,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
        );
      },
    );
  }
}
