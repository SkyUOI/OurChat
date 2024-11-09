fn main() -> anyhow::Result<()> {
    tonic_build::configure()
        .type_attribute(
            "msg_delivery.OneMsg",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "msg_delivery.OneMsg.data",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile_protos(&["../message/service.proto"], &[".."])?;
    Ok(())
}
