// file copyright release to public domain
#[macro_use]
extern crate wap;

wap_begin!(|global| {
    let console = wap::get(&global, "console").unwrap();
    let log = wap::get(&console, "log").unwrap();
    wap::call(&log, &["Hello World".to_string().into()]);
});

fn main() {}
