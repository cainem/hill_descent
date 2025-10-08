fn main() {
    // Link Windows libraries required by git2
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
    }
}
