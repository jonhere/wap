use std::mem;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::rc::{Rc, Weak};


//https://github.com/brson/mir2wasm/issues/33
//https://github.com/rust-lang/rust/issues/44006

//#[link(name = "env")]
extern "C" {
    //#[link_name="wap_get"]
    fn wap_get(instance: f64, from: f64, name: *mut u8, ret: *mut u8);
    fn wap_clone(index: f64) -> f64;
    fn wap_unmap(index: f64);
    fn wap_set_null(instance: f64, object: f64, name: *mut u8);
    fn wap_set_undefined(instance: f64, object: f64, name: *mut u8);
    fn wap_set_boolean(instance: f64, object: f64, name: *mut u8, val: bool);
    fn wap_set_number(instance: f64, object: f64, name: *mut u8, val: f64);
    fn wap_set_string(instance: f64, object: f64, name: *mut u8, ptr: *mut u8);
    fn wap_set_ref(instance: f64, object: f64, name: *mut u8, index: f64);
    fn wap_new_object() -> f64;
    fn wap_new_string(instance: f64, from: *mut u8) -> f64;
    fn wap_call(instance: f64, index_of_function: f64, num_args: u32, args: *mut u8, ret: *mut u8);
    fn wap_bound_call(
        instance: f64,
        index_of_object: f64,
        index_of_function: f64,
        num_args: u32,
        args: *mut u8,
        ret: *mut u8,
    );
    fn wap_instanceof(instance: f64, object: f64, of: *mut u8) -> bool;
    fn wap_delete(instance: f64, object: f64, name: *mut u8);
//fn wap new_boolean
//fn wap new_number
//fn wap new_construct
//fn wap_member_instanceof(instance: f64, object: f64, name: *mut u8, of: *mut u8,) -> bool;
//fn wap_typeof(object: f64) -> u32
//fn wap_member_typeof(instance: f64, object: f64, name: *mut u8) -> u32
}

// todo see if better as thread_local
static mut INSTANCE: f64 = 0.0;

#[derive(Clone)]
struct Index(f64);
#[derive(Clone)]
pub struct WapRc(Rc<Index>);
#[derive(Clone)]
pub struct WapWeak(Weak<Index>);

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

fn raw_instance() -> f64 {
    unsafe { INSTANCE }
}

pub fn instance() -> WapRc {
    let index = unsafe { wap_clone(raw_instance()) };
    WapRc::new(index)
}

/// Unmaps the instance which will allow JS to GC it.
/// WapRc are still safe to be dropped after calling this.
/// So long as no refs are holding it elsewhere.
// todo do i want unsafe?
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

fn wap_dealloc(ptr: *mut u8, cap: usize) {
    unsafe {
        //FORGOTTEN_MEM -= cap as isize;
        let _ = Vec::from_raw_parts(ptr, 0, cap);
    }
}

pub fn get(from: &WapRc, name: &str) -> JsType {
    let mut v = name.to_string().into_bytes();
    v.push(0);
    let name = v.as_mut_ptr();

    // byte 0 type
    // up to 8 for f64
    let mut a = unsafe { mem::uninitialized::<[u8; 9]>() };
    let ret8 = a.as_mut_ptr();

    unsafe {
        wap_get(raw_instance(), from.raw_index(), name, ret8);
    }

    let ret_type: RetTypes = unsafe { mem::transmute_copy(&*ret8) };

    match ret_type {
        RetTypes::Null => JsType::Null,
        RetTypes::Undefined => JsType::Undefined,
        RetTypes::Boolean => JsType::Boolean(unsafe { *ret8.offset(1) } != 0),
        RetTypes::Number => {
            let ret_f64: f64 = unsafe { *(ret8.offset(1) as *mut f64) };
            JsType::Number(ret_f64)
        }
        RetTypes::String => {
            let str_ptr = unsafe { *(ret8.offset(1) as *const *const c_char) };
            let d = unsafe { CStr::from_ptr(str_ptr) };
            let b = d.to_bytes();
            let alloc_len = b.len() + 1;
            let s = unsafe { String::from_utf8_unchecked(b.to_vec()) };
            wap_dealloc(str_ptr as *mut u8, alloc_len);
            JsType::String(s)
        }
        RetTypes::Ref => {
            let ret_f64: f64 = unsafe { *(ret8.offset(1) as *mut f64) };
            JsType::Ref(WapRc::new(ret_f64))
        }
    }
}

pub fn new_object() -> WapRc {
    let index = unsafe { wap_new_object() };
    WapRc::new(index)
}

pub fn new_string(text: &str) -> WapRc {
    let mut v = text.to_string().into_bytes();
    v.push(0);
    let text = v.as_mut_ptr();

    let index = unsafe { wap_new_string(raw_instance(), text) };
    WapRc::new(index)
}

pub fn set(object: &WapRc, name: &str, to: JsType) {
    let mut v = name.to_string().into_bytes();
    v.push(0);
    let name = v.as_mut_ptr();

    match to {
        JsType::Null => unsafe {
            wap_set_null(raw_instance(), object.raw_index(), name);
        },
        JsType::Undefined => unsafe {
            wap_set_undefined(raw_instance(), object.raw_index(), name);
        },
        JsType::Boolean(b) => unsafe {
            wap_set_boolean(raw_instance(), object.raw_index(), name, b);
        },
        JsType::Number(n) => unsafe {
            wap_set_number(raw_instance(), object.raw_index(), name, n);
        },
        JsType::String(s) => {
            let mut v = s.to_string().into_bytes();
            v.push(0);
            let s = v.as_mut_ptr();
            unsafe {
                wap_set_string(raw_instance(), object.raw_index(), name, s);
            }
        }
        JsType::Ref(r) => unsafe {
            wap_set_ref(raw_instance(), object.raw_index(), name, r.raw_index());
        },
    }
}

// todo move args Borrow/generic
pub fn call(function: &WapRc, args: &[JsType]) -> JsType {
    let num = args.len();

    let mut buf = vec![unsafe { mem::uninitialized() }; 9 * num];
    let args_ptr = buf.as_mut_ptr();

    let mut save = Vec::new();

    for (i, arg) in args.iter().enumerate() {
        match arg {
            &JsType::Null => {
                buf[9 * i] = 0;
            }
            &JsType::Undefined => {
                buf[9 * i] = 1;
            }
            &JsType::Boolean(b) => {
                buf[9 * i] = 2;
                buf[9 * i + 1] = b as u8;
            }
            &JsType::Number(n) => {
                buf[9 * i] = 3;
                unsafe {
                    *(args_ptr.offset(9 * i as isize + 1) as *mut f64) = n;
                }
            }
            &JsType::String(ref s) => {
                buf[9 * i] = 4;
                let mut v = s.clone().into_bytes();
                v.push(0);
                let p = v.as_mut_ptr();
                save.push(v);
                unsafe {
                    *(args_ptr.offset(9 * i as isize + 1) as *mut *mut u8) = p as *mut u8;
                }
            }
            &JsType::Ref(ref r) => {
                buf[9 * i] = 5;
                unsafe {
                    *(args_ptr.offset(9 * i as isize + 1) as *mut f64) = r.raw_index();
                }
            }
        }
    }

    // byte 0 type
    // up to 8 for f64
    let mut a = unsafe { mem::uninitialized::<[u8; 9]>() };
    let ret8 = a.as_mut_ptr();

    unsafe {
        wap_call(
            raw_instance(),
            function.raw_index(),
            num as u32,
            args_ptr,
            ret8,
        );
    }

    let ret_type: RetTypes = unsafe { mem::transmute_copy(&*ret8) };

    match ret_type {
        RetTypes::Null => JsType::Null,
        RetTypes::Undefined => JsType::Undefined,
        RetTypes::Boolean => JsType::Boolean(unsafe { *ret8.offset(1) } != 0),
        RetTypes::Number => {
            let ret_f64: f64 = unsafe { *(ret8.offset(1) as *mut f64) };
            JsType::Number(ret_f64)
        }
        RetTypes::String => {
            let ret_u32: u32 = unsafe { *(ret8.offset(1) as *mut u32) };
            let d = unsafe { CStr::from_ptr(ret_u32 as *const c_char) };
            let b = d.to_bytes();
            let alloc_len = b.len() + 1;
            let s = unsafe { String::from_utf8_unchecked(b.to_vec()) };
            wap_dealloc(ret_u32 as *mut u8, alloc_len);
            JsType::String(s)
        }
        RetTypes::Ref => {
            let ret_f64: f64 = unsafe { *(ret8.offset(1) as *mut f64) };
            JsType::Ref(WapRc::new(ret_f64))
        }
    }
}

// almost identical code copy of call()
pub fn bound_call(object: &WapRc, function: &WapRc, args: &[JsType]) -> JsType {
    let num = args.len();

    let mut buf = vec![unsafe { mem::uninitialized() }; 9 * num];
    let args_ptr = buf.as_mut_ptr();

    let mut save = Vec::new();

    for (i, arg) in args.iter().enumerate() {
        match arg {
            &JsType::Null => {
                buf[9 * i] = 0;
            }
            &JsType::Undefined => {
                buf[9 * i] = 1;
            }
            &JsType::Boolean(b) => {
                buf[9 * i] = 2;
                buf[9 * i + 1] = b as u8;
            }
            &JsType::Number(n) => {
                buf[9 * i] = 3;
                unsafe {
                    *(args_ptr.offset(9 * i as isize + 1) as *mut f64) = n;
                }
            }
            &JsType::String(ref s) => {
                buf[9 * i] = 4;
                let mut v = s.clone().into_bytes();
                v.push(0);
                let p = v.as_mut_ptr();
                save.push(v);
                unsafe {
                    *(args_ptr.offset(9 * i as isize + 1) as *mut *mut u8) = p;
                }
            }
            &JsType::Ref(ref r) => {
                buf[9 * i] = 5;
                unsafe {
                    *(args_ptr.offset(9 * i as isize + 1) as *mut f64) = r.raw_index();
                }
            }
        }
    }

    // byte 0 type
    // up to 8 for f64
    let mut a = unsafe { mem::uninitialized::<[u8; 9]>() };
    let ret8 = a.as_mut_ptr();

    unsafe {
        wap_bound_call(
            raw_instance(),
            object.raw_index(),
            function.raw_index(),
            num as u32,
            args_ptr,
            ret8,
        );
    }

    let ret_type: RetTypes = unsafe { mem::transmute_copy(&*ret8) };

    match ret_type {
        RetTypes::Null => JsType::Null,
        RetTypes::Undefined => JsType::Undefined,
        RetTypes::Boolean => JsType::Boolean(unsafe { *ret8.offset(1) } != 0),
        RetTypes::Number => {
            let ret_f64: f64 = unsafe { *(ret8.offset(1) as *mut f64) };
            JsType::Number(ret_f64)
        }
        RetTypes::String => {
            let str_ptr = unsafe { *(ret8.offset(1) as *const *const c_char) };
            let d = unsafe { CStr::from_ptr(str_ptr) };
            let b = d.to_bytes();
            let alloc_len = b.len() + 1;
            let s = unsafe { String::from_utf8_unchecked(b.to_vec()) };
            wap_dealloc(str_ptr as *mut u8, alloc_len);
            JsType::String(s)
        }
        RetTypes::Ref => {
            let ret_f64: f64 = unsafe { *(ret8.offset(1) as *mut f64) };
            JsType::Ref(WapRc::new(ret_f64))
        }
    }
}

pub fn instanceof(item: &WapRc, of: &str) -> bool {
    let mut v = of.to_string().into_bytes();
    v.push(0);
    let of = v.as_mut_ptr();

    unsafe { wap_instanceof(raw_instance(), item.raw_index(), of) }
}

pub fn delete(object: &WapRc, name: &str) {
    let mut v = name.to_string().into_bytes();
    v.push(0);
    let name = v.as_mut_ptr();

    unsafe {
        wap_delete(raw_instance(), object.raw_index(), name);
    }
}

/// Not to be called directly.
/// Used by wap_begin macro.
pub unsafe fn wap_begin_init(instance: f64, global: f64) -> WapRc {
    INSTANCE = instance;
    WapRc::new(global)
}

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
