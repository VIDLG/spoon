use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let source = PathBuf::from("native").join("helper.c");
    let object = out_dir.join("helper.obj");
    let compiler = env::var("SPOON_VALIDATE_SPOON_CL").expect("SPOON_VALIDATE_SPOON_CL");
    let status = Command::new("cmd")
        .arg("/C")
        .arg(&compiler)
        .arg("/nologo")
        .arg("/c")
        .arg(source)
        .arg(format!("/Fo{}", object.display()))
        .status()
        .expect("spoon-cl compile");
    assert!(status.success(), "spoon-cl failed: {status:?}");
    println!("cargo:rustc-link-arg={}", object.display());
    println!("cargo:rerun-if-changed=native/helper.c");
}
