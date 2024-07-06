use std::process::Command;

fn main() {
    let shader_folder = format!("{}/{}", std::env::var("OUT_DIR").unwrap(), "shaders");
    ensure_generated_dirs(&shader_folder).unwrap();

    compile_d2d_shader(&shader_folder, "ps_5_0", "ThresholdPixelShader");
}

fn compile_d2d_shader(shader_folder: &str, profile: &str, file_stem: &str) {
    println!("cargo:rerun-if-changed=src/{}.hlsl", file_stem);
    let pdb_out_dir = { format!("{}/", shader_folder) };
    let mut lut_generation_command = Command::new("fxc");
    let status = lut_generation_command
        .args([
            "/Zi",
            "/Zss",
            "/T",
            profile,
            "/D",
            "D2D_FULL_SHADER",
            "/D",
            "D2D_ENTRY=main",
            "/E",
            "main",
            "/Fd",
            &pdb_out_dir,
            "/Fo",
            &format!("{}/{}.cso", shader_folder, file_stem),
            "/I",
            &get_windows_sdk_um_path(),
            &format!("src/{}.hlsl", file_stem),
        ])
        .status()
        .unwrap();
    assert!(status.success());
}

fn ensure_generated_dirs(shader_folder: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(shader_folder)
}

fn get_windows_sdk_um_path() -> String {
    let sdk_dir = std::env::var("WindowsSdkDir").unwrap();
    let sdk_version = std::env::var("WindowsSDKVersion").unwrap();
    format!("{}\\Include\\{}um", sdk_dir, sdk_version)
}
