use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    // remove build/wap-XXXXXXX/out
    let dest_path = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("wap.js");
    let wap_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_path = Path::new(&wap_dir).join("src").join("wap.js");
    std::fs::copy(&src_path, &dest_path).unwrap();
}
