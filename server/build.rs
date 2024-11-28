fn main() -> anyhow::Result<()> {
    tonic_build::configure()
        .type_attribute(
            "service.ourchat.msg_delivery.v1.OneMsg",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.msg_delivery.v1.OneMsg.data",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile_protos(&["../service/ourchat/v1/ourchat.proto"], &[".."])?;
    tonic_build::configure().compile_protos(&["../service/auth/v1/auth.proto"], &[".."])?;
    tonic_build::configure().compile_protos(&["../service/basic/v1/basic.proto"], &[".."])?;
    Ok(())
}
