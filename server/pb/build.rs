fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=../../service");
    tonic_build::configure()
        .type_attribute(
            "service.ourchat.msg_delivery.v1.OneMsg",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "service.ourchat.msg_delivery.v1.OneMsg.data",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
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
