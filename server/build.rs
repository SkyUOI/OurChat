fn main() -> anyhow::Result<()> {
    tonic_build::configure()
        // .enum_attribute(".", "#[derive(num_enum::TryFromPrimitive)]")
        .compile_protos(&["../message/service.proto"], &[".."])?;
    Ok(())
}
