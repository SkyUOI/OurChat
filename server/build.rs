fn main() -> anyhow::Result<()> {
    tonic_build::configure().compile_protos(&["../message/service.proto"], &[".."])?;
    Ok(())
}
