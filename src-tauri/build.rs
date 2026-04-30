fn main() {
    #[cfg(target_os = "macos")]
    {
        compile_calendar_bridge();
    }
    tauri_build::build()
}

#[cfg(target_os = "macos")]
fn compile_calendar_bridge() {
    use std::process::Command;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let swift_src = format!("{}/src/swift/CalendarBridge.swift", manifest_dir);
    let out_dir = format!("{}/resources", manifest_dir);
    let out_bin = format!("{}/calendar-bridge", out_dir);

    std::fs::create_dir_all(&out_dir).expect("failed to create resources dir");

    let status = Command::new("swiftc")
        .args([
            &swift_src,
            "-o",
            &out_bin,
            "-framework",
            "EventKit",
            "-framework",
            "Foundation",
        ])
        .status()
        .expect("swiftc not found — install Xcode Command Line Tools");

    if !status.success() {
        panic!("CalendarBridge.swift compilation failed");
    }

    println!("cargo:rerun-if-changed={}", swift_src);
}
