use slint_build::CompilerConfiguration;

fn main() {
    let cfg = {
        #[cfg(target_os = "linux")]
        {
            // Default is unstyled Qt, WTF
            CompilerConfiguration::new().with_style("fluent".into())
        }
        #[cfg(not(target_os = "linux"))]
        {
            CompilerConfiguration::new()
        }
    };
    slint_build::compile_with_config("ui/app-window.slint", cfg).expect("Slint build failed");
    println!("cargo:rerun-if-changed=assets/icon.rc");
    println!("cargo:rustc-link-arg-bins=assets/icon.res");
}
