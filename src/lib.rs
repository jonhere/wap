//! Wap library allows you to write a web page app exclusively in Rust.
//!


use std::{mem, slice, str};
use std::rc::{Rc, Weak};


//https://github.com/brson/mir2wasm/issues/33
//https://github.com/rust-lang/rust/issues/44006

#[link(name = "env")]
extern "C" {
    //#[link_name="wap_get"]
    fn wap_get(instance: f64, from: f64, name_ptr: *const u8, name_len: usize, ret: *mut f64)
        -> u8;
    fn wap_clone(index: f64) -> f64;
    fn wap_unmap(index: f64);
    fn wap_set_null(instance: f64, object: f64, name_ptr: *const u8, name_len: usize);
    fn wap_set_undefined(instance: f64, object: f64, name_ptr: *const u8, name_len: usize);
    fn wap_set_boolean(instance: f64, object: f64, name_ptr: *const u8, name_len: usize, val: bool);
    fn wap_set_number(instance: f64, object: f64, name_ptr: *const u8, name_len: usize, val: f64);
    fn wap_set_string(
        instance: f64,
        object: f64,
        name_ptr: *const u8,
        name_len: usize,
        val_ptr: *const u8,
        val_len: usize,
    );
    fn wap_set_ref(instance: f64, object: f64, name_ptr: *const u8, name_len: usize, index: f64);
    fn wap_new_object() -> f64;
    fn wap_new_string(instance: f64, from_ptr: *const u8, from_len: usize) -> f64;
    fn wap_call(
        instance: f64,
        index_of_function: f64,
        num_args: u32,
        args_types: *const u8,
        args: *const f64,
        ret: *mut f64,
    ) -> u8;
    fn wap_bound_call(
        instance: f64,
        index_of_object: f64,
        index_of_function: f64,
        num_args: u32,
        args_types: *const u8,
        args: *const f64,
        ret: *mut f64,
    ) -> u8;
    fn wap_instanceof(instance: f64, object: f64, of_ptr: *const u8, of_len: usize) -> bool;
    fn wap_delete(instance: f64, object: f64, name_ptr: *const u8, name_len: usize);
//fn wap new_boolean
//fn wap new_number
//fn wap new_construct
//fn wap_member_instanceof(instance: f64, object: f64, name: *const u8, of: *const u8,) -> bool;
//fn wap_typeof(object: f64) -> u8
//fn wap_member_typeof(instance: f64, object: f64, name: *const u8) -> u8
}

// todo see if better as thread_local
static mut INSTANCE: f64 = 0.0;

#[derive(Clone)]
struct Index(f64);

/// Rc reference count to JavaScripts exclusive types; which get references stored internally and dropped when finished with WapRc.
#[derive(Clone)]
pub struct WapRc(Rc<Index>);

/// Weak companion to WapRc
#[derive(Clone)]
pub struct WapWeak(Weak<Index>);

/// The main data communication type in and out of function calls.
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

impl Drop for Index {
    fn drop(&mut self) {
        unsafe { wap_unmap(self.0) };
    }
}

impl WapRc {
    fn new(index: f64) -> WapRc {
        WapRc(Rc::new(Index(index)))
    }
    pub fn downgrade(&self) -> WapWeak {
        WapWeak(Rc::downgrade(&self.0))
    }
    fn raw_index(&self) -> f64 {
        (*self.0).0
    }
}

impl WapWeak {
    pub fn new() -> WapWeak {
        WapWeak(Weak::new())
    }
    pub fn upgrade(&self) -> Option<WapRc> {
        self.0.upgrade().map(|rc| WapRc(rc))
    }
}

impl JsType {
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
    let index = unsafe { wap_clone(raw_instance()) };
    WapRc::new(index)
}

/// Unmaps the instance which will allow JS to GC it.
/// WapRc are still safe to be dropped after calling this.
/// So long as no refs are holding it elsewhere.
pub unsafe fn shutdown() {
    wap_unmap(INSTANCE);
    INSTANCE = 0.01;
}

//pub static mut FORGOTTEN_MEM: isize = 0;

// alloc helpers from https://www.hellorust.com/demos/sha1/index.html
// https://news.ycombinator.com/item?id=15780702
/// Not to be called directly.
/// Used by js boilerplate.
#[no_mangle]
pub unsafe extern "C" fn wap_alloc(size: usize) -> *mut u8 {
    //unsafe { FORGOTTEN_MEM += size as isize };
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr
}

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
        mem::transmute(wap_get(
            raw_instance(),
            from.raw_index(),
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
    let index = unsafe { wap_new_object() };
    WapRc::new(index)
}

pub fn new_string(text: &str) -> WapRc {
    let mut v = text.to_string().into_bytes();
    let text = v.as_mut_ptr();
    let len = v.len();

    let index = unsafe { wap_new_string(raw_instance(), text, len) };
    WapRc::new(index)
}

pub fn set(object: &WapRc, name: &str, to: JsType) {
    let mut v = name.to_string().into_bytes();
    let name = v.as_mut_ptr();
    let len = v.len();

    match to {
        JsType::Null => unsafe {
            wap_set_null(raw_instance(), object.raw_index(), name, len);
        },
        JsType::Undefined => unsafe {
            wap_set_undefined(raw_instance(), object.raw_index(), name, len);
        },
        JsType::Boolean(b) => unsafe {
            wap_set_boolean(raw_instance(), object.raw_index(), name, len, b);
        },
        JsType::Number(n) => unsafe {
            wap_set_number(raw_instance(), object.raw_index(), name, len, n);
        },
        JsType::String(s) => {
            let mut v = s.to_string().into_bytes();
            let s = v.as_mut_ptr();
            let s_len = v.len();
            unsafe {
                wap_set_string(raw_instance(), object.raw_index(), name, len, s, s_len);
            }
        }
        JsType::Ref(r) => unsafe {
            wap_set_ref(raw_instance(), object.raw_index(), name, len, r.raw_index());
        },
    }
}


pub fn call(function: &WapRc, args: &[JsType]) -> JsType {
    let mut save = Vec::new();
    let (mut at_buf, mut buf) = args.into_iter()
        .map(|arg| match arg {
            &JsType::Null => (RetTypes::Null as u8, unsafe { mem::uninitialized() }),
            &JsType::Undefined => (RetTypes::Undefined as u8, unsafe {
                mem::uninitialized()
            }),
            &JsType::Boolean(b) => (RetTypes::Boolean as u8, if b { 1.0 } else { 0.0 }),
            &JsType::Number(n) => (RetTypes::Number as u8, n),
            &JsType::String(ref s) => {
                let mut v = s.clone().into_bytes();
                let p = v.as_ptr();
                let len = v.len();
                save.push(v);
                let mut f = unsafe { mem::uninitialized() };
                unsafe {
                    *(&mut f as *mut f64 as *mut *const u8) = p;
                    *(&mut f as *mut f64 as *mut usize).offset(1) = len;
                };
                (RetTypes::String as u8, f)
            }
            &JsType::Ref(ref r) => (RetTypes::Ref as u8, r.raw_index()),
        })
        .unzip::<_, _, Vec<u8>, Vec<f64>>();
    let num_args = at_buf.len();
    let args_types_ptr = at_buf.as_mut_ptr();
    let args_ptr = buf.as_mut_ptr();

    let mut ret64 = unsafe { mem::uninitialized::<f64>() };
    let ret_type: RetTypes = unsafe {
        mem::transmute(wap_call(
            raw_instance(),
            function.raw_index(),
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

// almost identical code copy of call()
pub fn bound_call(object: &WapRc, function: &WapRc, args: &[JsType]) -> JsType {
    let num_args = args.len();

    let mut at_buf = vec![unsafe { mem::uninitialized() }; num_args];
    let args_types_ptr = at_buf.as_mut_ptr();

    let mut buf = vec![unsafe { mem::uninitialized() }; num_args];
    let args_ptr = buf.as_mut_ptr();

    let mut save = Vec::new();

    for (i, arg) in args.iter().enumerate() {
        match arg {
            &JsType::Null => {
                at_buf[i] = RetTypes::Null as u8;
            }
            &JsType::Undefined => {
                at_buf[i] = RetTypes::Undefined as u8;
            }
            &JsType::Boolean(b) => {
                at_buf[i] = RetTypes::Boolean as u8;
                buf[i] = if b { 1.0 } else { 0.0 };
            }
            &JsType::Number(n) => {
                at_buf[i] = RetTypes::Number as u8;
                buf[i] = n;
            }
            &JsType::String(ref s) => {
                at_buf[i] = RetTypes::String as u8;
                let mut v = s.clone().into_bytes();
                let p = v.as_ptr();
                let len = v.len();
                save.push(v);
                unsafe {
                    *(&mut buf[i] as *mut f64 as *mut *const u8) = p;
                    *(&mut buf[i] as *mut f64 as *mut usize).offset(1) = len;
                }
            }
            &JsType::Ref(ref r) => {
                at_buf[i] = RetTypes::Ref as u8;
                buf[i] = r.raw_index();
            }
        }
    }

    let mut ret64 = unsafe { mem::uninitialized::<f64>() };
    let ret_type: RetTypes = unsafe {
        mem::transmute(wap_bound_call(
            raw_instance(),
            object.raw_index(),
            function.raw_index(),
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

pub fn instanceof(item: &WapRc, of: &str) -> bool {
    let mut v = of.to_string().into_bytes();
    let of = v.as_mut_ptr();
    let len = v.len();

    unsafe { wap_instanceof(raw_instance(), item.raw_index(), of, len) }
}

pub fn delete(object: &WapRc, name: &str) {
    let mut v = name.to_string().into_bytes();
    let name = v.as_mut_ptr();
    let len = v.len();

    unsafe {
        wap_delete(raw_instance(), object.raw_index(), name, len);
    }
}

/// Not to be called directly.
/// Used by wap_begin macro.
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
