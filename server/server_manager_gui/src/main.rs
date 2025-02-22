slint::include_modules!();

fn main() -> anyhow::Result<()> {
    let main_window = ServerManagerMainWindow::new()?;

    main_window.run()?;
    Ok(())
}
