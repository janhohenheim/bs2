slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let main_window = App::new()?;

    main_window.run()
}