fn main() -> anyhow::Result<()> {
    slint_build::compile("ui/app-window.slint")?;
    Ok(())
}
