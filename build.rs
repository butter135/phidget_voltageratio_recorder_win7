use std::env;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("app.manifest");

    println!("cargo:rerun-if-changed=app.manifest");

    println!("cargo:rustc-link-arg-bin=phidget_recorder_win7=/MANIFEST:EMBED");

    println!(
        "cargo:rustc-link-arg-bin=phidget_recorder_win7=/MANIFESTINPUT:{}",
        manifest.display()
    );
}
