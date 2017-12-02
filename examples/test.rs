#[macro_use]
extern crate wap;

use wap::*;
use std::cell::RefCell;

wap_begin!(|window| {
    assert!(wap::instanceof(&window, "Window"));
    let document = get(&window, "document").unwrap();
    let body = get(&document, "body").unwrap();
    let _s = new_string("hello");
    let _o = new_object();
    set(
        &body,
        "innerHTML",
        "<h1>Hello World</h1>".to_string().into(),
    );
    let _inner_html = get(&body, "innerHTML").unwrap_string();

    set(&window, "test", JsType::Boolean(true));
    delete(&window, "test");

    let eval = get(&window, "eval").unwrap();
    let random = call(&eval, &["Math.random".to_string().into()]).unwrap();
    let random = call(&random, &[]).unwrap_number();
    call(
        &eval,
        &[
            format!(
                "alert(\"Eval called.\\nOk to begin RAF loop\\n{}\");",
                random
            ).into(),
        ],
    );

    let instance = webassembly_instance();
    let exports = get(&instance, "exports").unwrap();
    let fn_loop = get(&exports, "fn_loop").unwrap();
    let raf = get(&window, "requestAnimationFrame").unwrap();
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.raf = Some(raf.clone());
        s.body = Some(body);
        s.fn_loop = Some(fn_loop.clone());
    });
    call(&raf, &[JsType::Ref(fn_loop)]);
});

struct State {
    raf: Option<WapRc>,
    fn_loop: Option<WapRc>,
    body: Option<WapRc>,
    count: u32,
    start: f64,
    last: f64,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State{
        raf: None,
        body: None,
        fn_loop: None,
        count: 0,
        start: std::f64::NAN,
        last: std::f64::NAN,
    });
}

#[no_mangle]
pub extern "C" fn fn_loop(time: f64) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let last = s.last;
        s.last = time;
        s.count += 1;

        if s.count == 1 {
            s.start = time;
        }

        let will_shutdown = if time.is_nan() || time < 10_000.0 {
            call(
                s.raf.as_ref().unwrap(),
                &[JsType::Ref(s.fn_loop.as_ref().unwrap().clone())],
            );
            false
        } else {
            true
        };

        set(
            s.body.as_ref().unwrap(),
            "innerText",
            JsType::String(
                "Hello World ".to_string() + &format!("{:.3}", s.start) + " " + &s.count.to_string()
                    + " " + &format!("{:.3}", time) + " "
                    + &format!("{:.3}", (time - last)),
            ),
        );

        if will_shutdown {
            // test if mem is freed
            let big_vec = vec![0u8; 500_000_000];
            s.raf = None;
            s.body = None;
            s.fn_loop = None;
            unsafe {
                shutdown();
            }
            std::mem::forget(big_vec);
        }
    });
}

fn main() {}
