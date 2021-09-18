fn main() {
    println!("cargo:rustc-link-search=font");
    println!("cargo:rerun-if-changed=font/hankaku.txt");

    std::process::Command::new("./makelib.sh")
        .current_dir("font")
        .status()
        .unwrap();
}
