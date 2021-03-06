//! Wap library allows you to write a web page (or node.js) app exclusively in Rust.
//!

use std::rc::{Rc, Weak};
use std::{mem, slice, str};

// see if this progresses https://github.com/rust-lang/rust/commit/6741e416feb54b18de41c348ecc70ba5cbc961ce
// or internal #![feature(set_stdio)]

mod wap_imp {
    #[link(wasm_import_module = "WapImp")]
    extern "C" {
        //#[link_name = "get"]
        pub fn get(
            instance: f64,
            from: f64,
            name_ptr: *const u8,
            name_len: usize,
            ret: *mut f64,
        ) -> u8;
        pub fn clone(handle: f64) -> f64;
        pub fn unmap(handle: f64);
        pub fn set_null(instance: f64, object: f64, name_ptr: *const u8, name_len: usize);
        pub fn set_undefined(instance: f64, object: f64, name_ptr: *const u8, name_len: usize);
        pub fn set_boolean(
            instance: f64,
            object: f64,
            name_ptr: *const u8,
            name_len: usize,
            val: bool,
        );
        pub fn set_number(
            instance: f64,
            object: f64,
            name_ptr: *const u8,
            name_len: usize,
            val: f64,
        );
        pub fn set_string(
            instance: f64,
            object: f64,
            name_ptr: *const u8,
            name_len: usize,
            val_ptr: *const u8,
            val_len: usize,
        );
        pub fn set_ref(
            instance: f64,
            object: f64,
            name_ptr: *const u8,
            name_len: usize,
            handle: f64,
        );
        pub fn new_object() -> f64;
        pub fn new_string(instance: f64, from_ptr: *const u8, from_len: usize) -> f64;
        pub fn new_construct(
            instance: f64,
            constructor_handle: f64,
            num_args: u32,
            args_types: *const u8,
            args: *const f64,
        ) -> f64;
        pub fn call(
            instance: f64,
            handle_of_function: f64,
            num_args: u32,
            args_types: *const u8,
            args: *const f64,
            ret: *mut f64,
        ) -> u8;
        pub fn bound_call(
            instance: f64,
            handle_of_object: f64,
            handle_of_function: f64,
            num_args: u32,
            args_types: *const u8,
            args: *const f64,
            ret: *mut f64,
        ) -> u8;
        pub fn instanceof(instance: f64, object: f64, constructor: f64) -> bool;
        pub fn delete(instance: f64, object: f64, name_ptr: *const u8, name_len: usize);
        pub fn eq(first: f64, second: f64) -> bool;
    //fn new_boolean - no practical use storing it in wap
    //fn new_number - avoid to keep open for complex allocation; see new_handle function
    //fn typeof(object: f64) -> u8 - should know when get/new_strig called so don't see need
    //fn member_typeof(instance: f64, object: f64, name: *const u8) -> u8 - just use get
    }
}
// todo see if better as thread_local
static mut INSTANCE: f64 = 0.0;

#[derive(Debug, Clone)]
struct Handle(f64);

/// Rc reference count to JavaScripts exclusive types; which get references stored internally and dropped when finished with WapRc.
#[derive(Debug, Clone)]
pub struct WapRc(Rc<Handle>);

/// Weak companion to WapRc
#[derive(Debug, Clone)]
pub struct WapWeak(Weak<Handle>);

/// The main data communication type in and out of function calls.
#[derive(Debug, Clone)]
pub enum JsType {
    Null,
    Undefined,
    Boolean(bool),
    Number(f64),
    String(String),
    /// Object/function/Symbol
    Ref(WapRc),
}

// hide "warning: variant is never constructed:"
// compiler misses transmute
#[allow(dead_code)]
enum RetTypes {
    Null = 0,
    Undefined = 1,
    Boolean = 2,
    Number = 3,
    String = 4,
    Ref = 5,
}

/// Prettier interface to the functions
#[macro_export]
macro_rules! w {
    ({}) => {
        $crate::new_object()
    };
    (string $s:expr ) => {
        $crate::new_string($s.as_ref())
    };
    (new $f:ident( $( $a:expr ),*  )) => {
        $crate::new_construct(&$f, &[
            $(
                $a.into(),
            )*
        ])
    };
    ($o:ident.$f:ident( $( $a:expr ),* )) => {
        $crate::bound_call(&$o, &$f, &[
            $(
                $a.into(),
            )*
        ])
    };
    ($f:ident( $( $a:expr ),* )) => {
        $crate::call(&$f, &[
            $(
                $a.into(),
            )*
        ])
    };
    ($o:ident instanceof $f:ident) => {
        $crate::instanceof(&$o, &$f)
    };
    (delete $o:ident[$n:tt]) => {
        $crate::delete(&$o, $n.as_ref())
    };
    ($o:ident[$n:tt] = $v:expr ) => {
        $crate::set(&$o, $n.as_ref(), $v.into())
    };
    ($o:ident[$n:tt]) => {
        $crate::get(&$o, $n.as_ref())
    };
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { wap_imp::unmap(self.0) };
    }
}

impl WapRc {
    fn new(handle: f64) -> WapRc {
        WapRc(Rc::new(Handle(handle)))
    }
    pub fn downgrade(&self) -> WapWeak {
        WapWeak(Rc::downgrade(&self.0))
    }
    fn raw_handle(&self) -> f64 {
        (*self.0).0
    }
}

impl std::cmp::PartialEq for WapRc {
    fn eq(&self, other: &Self) -> bool {
        if self.raw_handle() == other.raw_handle() {
            return true;
        }
        unsafe { wap_imp::eq(self.raw_handle(), other.raw_handle()) }
    }
}

#[allow(clippy::new_without_default_derive)]
impl WapWeak {
    pub fn new() -> WapWeak {
        WapWeak(Weak::new())
    }
    pub fn upgrade(&self) -> Option<WapRc> {
        self.0.upgrade().map(WapRc)
    }
}

impl JsType {
    pub fn is_null(&self) -> bool {
        match self {
            JsType::Null => true,
            _ => false,
        }
    }

    pub fn is_undefined(&self) -> bool {
        match self {
            JsType::Undefined => true,
            _ => false,
        }
    }

    pub fn unwrap(self) -> WapRc {
        match self {
            JsType::Ref(r) => r,
            _ => panic!("JsType not a Ref"),
        }
    }
    pub fn unwrap_string(self) -> String {
        match self {
            JsType::String(s) => s,
            _ => panic!("JsType not a String"),
        }
    }
    pub fn unwrap_number(self) -> f64 {
        match self {
            JsType::Number(n) => n,
            _ => panic!("JsType not a Number"),
        }
    }
    pub fn unwrap_boolean(self) -> bool {
        match self {
            JsType::Boolean(b) => b,
            _ => panic!("JsType not a Boolean"),
        }
    }
}

impl From<bool> for JsType {
    fn from(b: bool) -> Self {
        JsType::Boolean(b)
    }
}

impl From<f64> for JsType {
    fn from(n: f64) -> Self {
        JsType::Number(n)
    }
}

impl From<String> for JsType {
    fn from(s: String) -> Self {
        JsType::String(s)
    }
}

impl From<WapRc> for JsType {
    fn from(r: WapRc) -> Self {
        JsType::Ref(r)
    }
}

fn raw_instance() -> f64 {
    unsafe { INSTANCE }
}

pub fn webassembly_instance() -> WapRc {
    let handle = unsafe { wap_imp::clone(raw_instance()) };
    WapRc::new(handle)
}

/// Unmaps the instance which will allow JS to GC it.
/// WapRc are still safe to be dropped after calling this.
/// So long as no refs are holding it elsewhere.
pub unsafe fn shutdown() {
    wap_imp::unmap(INSTANCE);
    INSTANCE = 0.01;
}

//pub static mut FORGOTTEN_MEM: isize = 0;

// alloc helpers from https://www.hellorust.com/demos/sha1/index.html
// https://news.ycombinator.com/item?id=15780702
/// Not to be called directly.
/// Used by js boilerplate.
#[no_mangle]
#[doc(hidden)]
pub unsafe extern "C" fn wap_alloc(size: usize) -> *mut u8 {
    //unsafe { FORGOTTEN_MEM += size as isize };
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr
}

#[doc(hidden)]
unsafe fn wap_dealloc(ptr: *mut u8, cap: usize) {
    //FORGOTTEN_MEM -= cap as isize;
    let _ = Vec::from_raw_parts(ptr, 0, cap);
}

pub fn get(from: &WapRc, name: &str) -> JsType {
    let mut v = name.to_string().into_bytes();
    let name = v.as_mut_ptr();
    let len = v.len();

    let mut ret64 = unsafe { mem::uninitialized::<f64>() };
    let ret_type: RetTypes = unsafe {
        mem::transmute(wap_imp::get(
            raw_instance(),
            from.raw_handle(),
            name,
            len,
            &mut ret64 as *mut f64,
        ))
    };

    match ret_type {
        RetTypes::Null => JsType::Null,
        RetTypes::Undefined => JsType::Undefined,
        RetTypes::Boolean => JsType::Boolean(ret64 != 0.0),
        RetTypes::Number => JsType::Number(ret64),
        RetTypes::String => {
            let s = unsafe {
                let ptr = *(&ret64 as *const f64 as *const *mut u8);
                let len = *(&ret64 as *const f64 as *const usize).offset(1);
                let slice = slice::from_raw_parts(ptr, len);
                let s = str::from_utf8_unchecked(slice).to_owned();
                wap_dealloc(ptr, len);
                s
            };
            JsType::String(s)
        }
        RetTypes::Ref => JsType::Ref(WapRc::new(ret64)),
    }
}

pub fn new_object() -> WapRc {
    let handle = unsafe { wap_imp::new_object() };
    WapRc::new(handle)
}

pub fn new_string(text: &str) -> WapRc {
    let mut v = text.to_string().into_bytes();
    let text = v.as_mut_ptr();
    let len = v.len();

    let handle = unsafe { wap_imp::new_string(raw_instance(), text, len) };
    WapRc::new(handle)
}

type Persist = (Vec<Vec<u8>>, Vec<u8>, Vec<f64>);
fn raw_args(args: &[JsType]) -> (Persist, u32, *const u8, *const f64) {
    let mut persist_string_bytes = Vec::new();
    let (at_buf, buf) = args
        .iter()
        .map(|arg| match arg {
            JsType::Null => (RetTypes::Null as u8, unsafe { mem::uninitialized() }),
            JsType::Undefined => (RetTypes::Undefined as u8, unsafe { mem::uninitialized() }),
            &JsType::Boolean(b) => (RetTypes::Boolean as u8, if b { 1.0 } else { 0.0 }),
            &JsType::Number(n) => (RetTypes::Number as u8, n),
            JsType::String(s) => {
                let v = s.clone().into_bytes();
                let p = v.as_ptr();
                let len = v.len();
                persist_string_bytes.push(v);
                let mut f = unsafe { mem::uninitialized() };
                unsafe {
                    *(&mut f as *mut f64 as *mut *const u8) = p;
                    *(&mut f as *mut f64 as *mut usize).offset(1) = len;
                };
                (RetTypes::String as u8, f)
            }
            JsType::Ref(r) => (RetTypes::Ref as u8, r.raw_handle()),
        })
        .unzip::<_, _, Vec<u8>, Vec<f64>>();

    let num_args = at_buf.len() as u32;
    let args_types_ptr = at_buf.as_ptr();
    let args_ptr = buf.as_ptr();

    (
        (persist_string_bytes, at_buf, buf),
        num_args,
        args_types_ptr,
        args_ptr,
    )
}

pub fn new_construct(constructor: &WapRc, args: &[JsType]) -> WapRc {
    let (_persist, num_args, args_types_ptr, args_ptr) = raw_args(args);

    let handle = unsafe {
        wap_imp::new_construct(
            raw_instance(),
            constructor.raw_handle(),
            num_args,
            args_types_ptr,
            args_ptr,
        )
    };
    WapRc::new(handle)
}

pub fn set(object: &WapRc, name: &str, to: JsType) {
    let mut v = name.to_string().into_bytes();
    let name = v.as_mut_ptr();
    let len = v.len();

    match to {
        JsType::Null => unsafe {
            wap_imp::set_null(raw_instance(), object.raw_handle(), name, len);
        },
        JsType::Undefined => unsafe {
            wap_imp::set_undefined(raw_instance(), object.raw_handle(), name, len);
        },
        JsType::Boolean(b) => unsafe {
            wap_imp::set_boolean(raw_instance(), object.raw_handle(), name, len, b);
        },
        JsType::Number(n) => unsafe {
            wap_imp::set_number(raw_instance(), object.raw_handle(), name, len, n);
        },
        JsType::String(s) => {
            let mut v = s.to_string().into_bytes();
            let s = v.as_mut_ptr();
            let s_len = v.len();
            unsafe {
                wap_imp::set_string(raw_instance(), object.raw_handle(), name, len, s, s_len);
            }
        }
        JsType::Ref(r) => unsafe {
            wap_imp::set_ref(raw_instance(), object.raw_handle(), name, len, r.raw_handle());
        },
    }
}

pub fn call(function: &WapRc, args: &[JsType]) -> JsType {
    let (_persist, num_args, args_types_ptr, args_ptr) = raw_args(args);

    let mut ret64 = unsafe { mem::uninitialized::<f64>() };
    let ret_type: RetTypes = unsafe {
        mem::transmute(wap_imp::call(
            raw_instance(),
            function.raw_handle(),
            num_args,
            args_types_ptr,
            args_ptr,
            &mut ret64 as *mut f64,
        ))
    };

    match ret_type {
        RetTypes::Null => JsType::Null,
        RetTypes::Undefined => JsType::Undefined,
        RetTypes::Boolean => JsType::Boolean(ret64 != 0.0),
        RetTypes::Number => JsType::Number(ret64),
        RetTypes::String => {
            let s = unsafe {
                let ptr = *(&ret64 as *const f64 as *const *mut u8);
                let len = *(&ret64 as *const f64 as *const usize).offset(1);
                let slice = slice::from_raw_parts(ptr, len);
                let s = str::from_utf8_unchecked(slice).to_owned();
                wap_dealloc(ptr, len);
                s
            };
            JsType::String(s)
        }
        RetTypes::Ref => JsType::Ref(WapRc::new(ret64)),
    }
}

// almost identical code copy of call()
pub fn bound_call(object: &WapRc, function: &WapRc, args: &[JsType]) -> JsType {
    let (_persist, num_args, args_types_ptr, args_ptr) = raw_args(args);

    let mut ret64 = unsafe { mem::uninitialized::<f64>() };
    let ret_type: RetTypes = unsafe {
        mem::transmute(wap_imp::bound_call(
            raw_instance(),
            object.raw_handle(),
            function.raw_handle(),
            num_args as u32,
            args_types_ptr,
            args_ptr,
            &mut ret64 as *mut f64,
        ))
    };

    match ret_type {
        RetTypes::Null => JsType::Null,
        RetTypes::Undefined => JsType::Undefined,
        RetTypes::Boolean => JsType::Boolean(ret64 != 0.0),
        RetTypes::Number => JsType::Number(ret64),
        RetTypes::String => {
            let s = unsafe {
                let ptr = *(&ret64 as *const f64 as *const *mut u8);
                let len = *(&ret64 as *const f64 as *const usize).offset(1);
                let slice = slice::from_raw_parts(ptr, len);
                let s = str::from_utf8_unchecked(slice).to_owned();
                wap_dealloc(ptr, len);
                s
            };
            JsType::String(s)
        }
        RetTypes::Ref => JsType::Ref(WapRc::new(ret64)),
    }
}

pub fn instanceof(item: &WapRc, constructor: &WapRc) -> bool {
    unsafe { wap_imp::instanceof(raw_instance(), item.raw_handle(), constructor.raw_handle()) }
}

pub fn delete(object: &WapRc, name: &str) {
    let mut v = name.to_string().into_bytes();
    let name = v.as_mut_ptr();
    let len = v.len();

    unsafe {
        wap_imp::delete(raw_instance(), object.raw_handle(), name, len);
    }
}

/// Not to be called directly.
/// Used by wap_begin macro.
#[doc(hidden)]
pub unsafe fn wap_begin_init(instance: f64, global: f64) -> WapRc {
    INSTANCE = instance;
    WapRc::new(global)
}

/// Starting point from the boilerplait wap.js into the wasm. Takes a function pointer whos argument is
/// a WapRc to JavaScripts global object.
#[macro_export]
macro_rules! wap_begin {
    ($fn:expr) => {
        #[no_mangle]
        pub extern "C" fn wap_begin(instance: f64, global: f64) {
            assert_eq!(::std::mem::size_of::<usize>(), 4);
            assert_eq!(::std::mem::size_of::<*mut u8>(), 4);
            assert_eq!(::std::mem::size_of::<*mut std::os::raw::c_void>(), 4);
            //assert_eq!(::std::mem::size_of::<c_char>(), 1);

            let global = unsafe { $crate::wap_begin_init(instance, global) };

            let f = $fn;
            f(global);
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
