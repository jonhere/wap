#![allow(clippy::cyclomatic_complexity)]
use wap::*;

wap_begin!(|global| {
    //let console = wap::get(&global, "console").unwrap();
    let console = w!(global["console"]).unwrap();
    //let log = wap::get(&console, "log").unwrap();
    let log = w!(console["log"]).unwrap();
    //let eval = wap::get(&global, "eval").unwrap();
    let eval = w!(global["eval"]).unwrap();

    let wai = webassembly_instance();
    //let to = new_object();
    let to = w!({});
    //let test_str = new_string("test string");
    let test_str = w!(string "test string");

    //let c_object = get(&global, "Object").unwrap();
    let c_object = w!(global["Object"]).unwrap();
    //let _ = new_construct(&c_object, &[]);
    let _ = w!(new c_object());
    /*
    let confn = call(
        &eval,
        &[
            "let f = function(insout) { this.member = insout; }; f"
                .to_string()
                .into(),
        ],
    ).unwrap();
    */
    let confn = w!(eval(
        "let f = function(insout) { this.member = insout; }; f".to_string()
    ))
    .unwrap();

    //let cono = new_construct(&confn, &["testc".to_string().into()]);
    let cono = w!(new confn("testc".to_string()));
    //assert_eq!(get(&cono, "member").unwrap_string(), "testc");
    assert_eq!(w!(cono["member"]).unwrap_string(), "testc");

    //set(&to, "isanull", JsType::Null);
    w!(to["isanull"] = JsType::Null);
    //assert!(get(&to, "isanull").is_null());
    assert!(w!(to["isanull"]).is_null());
    //todo decide is want to use macro
    set(&to, "isundefined", JsType::Undefined);
    assert!(get(&to, "isundefined").is_undefined());
    set(&to, "abool", true.into());
    assert!(get(&to, "abool").unwrap_boolean());
    set(&to, "abool", false.into());
    assert!(!get(&to, "abool").unwrap_boolean());
    set(&to, "anumber", 43.0.into());
    #[allow(clippy::float_cmp)]
    {
        assert_eq!(get(&to, "anumber").unwrap_number(), 43.0);
    }
    set(&to, "astring", "wasm".to_string().into());
    assert_eq!(get(&to, "astring").unwrap_string(), "wasm");
    set(&to, "aselfref", to.clone().into());
    let _ = get(&to, "aselfref").unwrap();

    let myfn = call(
        &eval,
        &["let f = function(insout) { return insout; }; f"
            .to_string()
            .into()],
    )
    .unwrap();

    assert!(call(&myfn, &[JsType::Null]).is_null());
    assert!(call(&myfn, &[JsType::Undefined]).is_undefined());
    assert!(call(&myfn, &[]).is_undefined());
    assert!(call(&myfn, &[true.into()]).unwrap_boolean());
    #[allow(clippy::float_cmp)]
    {
        assert_eq!(call(&myfn, &[43.2.into()]).unwrap_number(), 43.2);
    }
    assert_eq!(
        call(&myfn, &["43".to_string().into()]).unwrap_string(),
        "43"
    );
    let _ = call(&myfn, &[to.clone().into()]).unwrap();

    let myfn = call(
        &eval,
        &["let f = function(a1,a2,insout) { return insout; }; f"
            .to_string()
            .into()],
    )
    .unwrap();
    assert!(call(&myfn, &["".to_string().into(), 43.0.into(), JsType::Null]).is_null());

    let myfn = call(
        &eval,
        &["let f = function(item) { return this[item]; }; f"
            .to_string()
            .into()],
    )
    .unwrap();

    assert!(bound_call(&to, &myfn, &["isanull".to_string().into()]).is_null());
    assert!(bound_call(&to, &myfn, &["isundefined".to_string().into()]).is_undefined());
    assert!(!bound_call(&to, &myfn, &["abool".to_string().into()]).unwrap_boolean());
    #[allow(clippy::float_cmp)]
    {
        assert_eq!(
            bound_call(&to, &myfn, &["anumber".to_string().into()]).unwrap_number(),
            43.0
        );
    }
    assert_eq!(
        bound_call(&to, &myfn, &["astring".to_string().into()]).unwrap_string(),
        "wasm"
    );
    let _ = bound_call(&to, &myfn, &["aselfref".to_string().into()]).unwrap();

    let c_function = get(&global, "Function").unwrap();
    assert!(instanceof(&to, &c_object));
    assert!(instanceof(&eval, &c_function));

    delete(&to, "isanull");
    assert!(get(&to, "isanull").is_undefined());

    assert_eq!(&wai, &webassembly_instance());
    assert_ne!(&wai, &to);
    assert_eq!(&test_str, &test_str.clone());
    assert_ne!(&test_str, &c_function);

    wap::call(
        &log,
        &["Tests Complete. (Finally shutdown)".to_string().into()],
    );
    unsafe {
        shutdown();
    }
});

fn main() {}
