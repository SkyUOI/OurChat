fn main() -> anyhow::Result<()> {
    shadow_rs::ShadowBuilder::builder().build()?;
    Ok(())
}
