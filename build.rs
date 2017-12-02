use std::env;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
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

    #[cfg(feature = "console-log")]
    std::fs::copy(&src_path, &dest_path).unwrap();

    #[cfg(not(feature = "console-log"))]
    {
        let src = File::open(&src_path).unwrap();
        let reader = BufReader::new(src);
        let out = File::create(&dest_path).unwrap();
        let mut writer = BufWriter::new(out);

        let mut multiline_debug = false;
        for line in reader.lines() {
            let line = line.unwrap();
            let trim = line.trim();
            if trim.starts_with("//") {
                continue;
            }
            if trim.starts_with("debug(") {
                if !trim.ends_with(");") {
                    multiline_debug = true;
                }
                continue;
            }
            if multiline_debug {
                if trim.ends_with(");") {
                    multiline_debug = false;
                }
                continue;
            }
            debug_assert!(!multiline_debug);

            writeln!(&mut writer, "{}", line).unwrap();
        }
    }
}
