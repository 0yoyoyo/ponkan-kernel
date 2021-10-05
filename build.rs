fn main() {
    println!("cargo:rustc-link-search=font");
    println!("cargo:rustc-link-search=driver");

    std::process::Command::new("make")
        .current_dir("font")
        .status()
        .unwrap();
    std::process::Command::new("make")
        .current_dir("driver")
        .output()
        .unwrap();
}
