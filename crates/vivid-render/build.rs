use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    println!("cargo:rerun-if-changed=shaders");
    println!("cargo:rerun-if-env-changed=SLANGC");

    let slangc = env::var("SLANGC").unwrap_or_else(|_| "slangc".to_owned());
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("cartgo sets OUT_DIR"));

    let mut compiled = 0u32;
    for entry in std::fs::read_dir("shaders").expect("vivid-render/shaders/ must exist") {
        let path = entry.expect("readable directory entry").path();
        if path.extension().is_none_or(|ext| ext != "slang") {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());

        let shader_wgsl = path
            .file_stem()
            .map(|stem| Path::new(stem).with_extension("wgsl"))
            .expect("shader file has a name");
        let out = out_dir.join(shader_wgsl);

        let status = Command::new(&slangc)
            .arg(&path)
            .args(["-target", "wgsl"])
            .arg("-o")
            .arg(&out)
            .status();

        match status {
            Ok(s) if s.success() => compiled += 1,
            Ok(s) => panic!("slangc failed on {} ({s})", path.display()),
            Err(e) => panic!(
                "could not run `{slangc}`: {e}\n\
                 install Slang (Vulkan SDK, or a shader-slang GitHub release)\
                 and put slangc on PATH, or set SLANGC=/path/to/slangc"
            ),
        }
    }

    assert!(compiled > 0, "no .slang files found in shaders/");
}
