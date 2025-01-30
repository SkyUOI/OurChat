use std::fs::{File, metadata, remove_file};
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir::WalkDir;

fn main() -> anyhow::Result<()> {
    let proto_files: Vec<_> = WalkDir::new("../../service")
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
    if !should_rebuilt {
        return Ok(());
    }
    tonic_build::configure()
        .type_attribute(
            "service.ourchat.msg_delivery.v1.FetchMsgsResponse.respond_msg_type",
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
            "service.ourchat.friends.accept_friend.v1.AcceptFriendNotification",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.session.invite_session.v1.InviteSession",
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
        .compile_well_known_types(true)
        .bytes(["."])
        .out_dir("./src/generated/")
        .compile_protos(
            &[
                "../../service/ourchat/v1/ourchat.proto",
                "../../service/auth/v1/auth.proto",
                "../../service/basic/v1/basic.proto",
            ],
            &["../.."],
        )?;
    Ok(())
}
