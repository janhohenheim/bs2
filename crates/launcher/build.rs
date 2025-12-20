fn main() {
    println!("cargo:rerun-if-changed=assets/icon.rc");
    println!("cargo:rustc-link-arg-bins=assets/icon.res");
}
