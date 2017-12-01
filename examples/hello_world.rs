// file copyright release to public domain
#[macro_use]
extern crate wap;

use wap::JsType;

wap_begin!(|window| {
    let document = wap::get(&window, "document").unwrap();
    let body = wap::get(&document, "body").unwrap();
    wap::set(
        &body,
        "innerHTML",
        JsType::String("<h1>Hello World</h1>".to_string()),
    );
});

fn main() {}
