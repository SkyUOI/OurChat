use std::env;
use std::fs::{File, metadata, remove_file};
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir::WalkDir;

fn main() -> anyhow::Result<()> {
    let mut proto_files: Vec<_> = WalkDir::new("../../service")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "proto")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_owned())
        .collect();
    for proto_file in &proto_files {
        println!("cargo:rerun-if-changed={}", proto_file.display());
    }
    let last_build = PathBuf::from("src/generated/.last_build");
    let last_modified = if !last_build.exists() {
        File::create(&last_build)?;
        SystemTime::UNIX_EPOCH
    } else {
        metadata(&last_build)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    };

    let mut should_rebuilt = false;
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let build_rs_path = manifest_dir.join("build.rs");
    proto_files.push(build_rs_path);
    for proto_file in &proto_files {
        let modified = metadata(proto_file)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        if modified > last_modified {
            remove_file(&last_build)?;
            File::create(&last_build)?;
            should_rebuilt = true;
            break;
        }
    }
    proto_files.pop();
    if !should_rebuilt {
        return Ok(());
    }
    tonic_prost_build::configure()
        .type_attribute(
            "service.ourchat.msg_delivery.v1.FetchMsgsResponse.respond_event_type",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.msg_delivery.v1.Msg",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.msg_delivery.v1.OneMsg",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.msg_delivery.v1.OneMsg.data",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.friends.accept_friend_invitation.v1.FriendInvitationResultNotification",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.session.invite_user_to_session.v1.InviteUserToSession",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.msg_delivery.recall.v1.RecallNotification",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "google.protobuf.Timestamp",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.session.join_session.v1.JoinSessionApproval",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.session.allow_user_join_session.v1.AllowUserJoinSessionNotification",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.friends.add_friend.v1.NewFriendInvitationNotification",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.friends.add_friend.v1.AddFriendRequest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.msg_delivery.announcement.v1.Announcement",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.msg_delivery.announcement.v1.AnnouncementResponse",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.session.invite_user_to_session.v1.AcceptSessionNotification",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.session.session_room_key.v1.ReceiveRoomKeyNotification",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.session.session_room_key.v1.SendRoomKeyNotification",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.session.session_room_key.v1.UpdateRoomKeyNotification",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile_well_known_types(true)
        .bytes(".")
        .out_dir("./src/generated/")
        .compile_protos(
            &[
                "../../service/ourchat/v1/ourchat.proto",
                "../../service/auth/v1/auth.proto",
                "../../service/basic/v1/basic.proto",
                "../../service/server_manage/v1/server_manage.proto",
            ],
            &["../.."],
        )?;
    Ok(())
}
