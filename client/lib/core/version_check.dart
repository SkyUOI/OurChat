import 'package:ourchat/service/basic/server/v1/server.pb.dart';

/// Lowest server version this client build can talk to (issue #16). Bump it
/// when a release starts relying on server APIs older servers do not have.
const ({int major, int minor, int patch}) minimumSupportedServerVersion = (
  major: 0,
  minor: 1,
  patch: 0,
);

/// Outcome of the bidirectional version negotiation (issue #16). The check is
/// advisory only — the user can always keep using the connection.
enum ServerCompatibility { ok, serverTooOld, clientTooOld }

/// Parse a version string like `v1.2.3.beta` into its numeric parts; anything
/// non-numeric (pre-release suffixes) is ignored. Returns null when no number
/// can be extracted at all.
({int major, int minor, int patch})? parseVersionString(String raw) {
  final numbers = RegExp(
    r'\d+',
  ).allMatches(raw).map((m) => int.parse(m.group(0)!)).toList();
  if (numbers.isEmpty) return null;
  return (
    major: numbers[0],
    minor: numbers.length > 1 ? numbers[1] : 0,
    patch: numbers.length > 2 ? numbers[2] : 0,
  );
}

int _compareVersions(
  ({int major, int minor, int patch}) a,
  ({int major, int minor, int patch}) b,
) {
  if (a.major != b.major) return a.major.compareTo(b.major);
  if (a.minor != b.minor) return a.minor.compareTo(b.minor);
  return a.patch.compareTo(b.patch);
}

/// Compare the negotiated versions both ways:
/// - the server must be at least [minimumSupportedServerVersion];
/// - the local client build must satisfy the server's advertised
///   `minimumClientVersion` (a `0.0.0` server setting means "no limit").
///
/// Missing info on either side (old server, dev build version) never blocks.
ServerCompatibility checkServerCompatibility({
  required ServerVersion? serverVersion,
  required ServerVersion? minimumClientVersion,
  required String clientVersion,
}) {
  if (serverVersion != null &&
      _compareVersions((
            major: serverVersion.major,
            minor: serverVersion.minor,
            patch: serverVersion.patch,
          ), minimumSupportedServerVersion) <
          0) {
    return ServerCompatibility.serverTooOld;
  }
  if (minimumClientVersionApplies(minimumClientVersion)) {
    final local = parseVersionString(clientVersion);
    if (local != null &&
        _compareVersions(local, (
              major: minimumClientVersion!.major,
              minor: minimumClientVersion.minor,
              patch: minimumClientVersion.patch,
            )) <
            0) {
      return ServerCompatibility.clientTooOld;
    }
  }
  return ServerCompatibility.ok;
}

/// A `0.0.0` (or absent) server-side minimum means "no client restriction".
bool minimumClientVersionApplies(ServerVersion? v) {
  if (v == null) return false;
  return v.major != 0 || v.minor != 0 || v.patch != 0;
}
