// file copyright release to public domain
'use strict';
{

  // note: ensure src calls comply with the simple stripping rule used in build.rs
  // i.e. "debug(" at start of line
  // ");" at end of any ending line
  // no single unenclosed if/loops calling debug
  const debug = function (s) {
    console.log(s);
  };

  const wap = new Map();

  // could start negative to double max range but small good to debug
  let next = 1;
  const new_handle = function () {
    // breaks using f64 at 2^53 (Number.MAX_SAFE_INTEGER) // todo test how long it would take to reach
    // since values are never number could switch code to more complex reusage of handles
    // storing the max and with next pointing chain of any freed handles
    return next++;
  }


  function get_text_decoder() {
    if (typeof TextDecoder == "function") {
      return new TextDecoder("UTF-8");
    } else {
      const util = require("util");
      const TextDecoder = util.TextDecoder;
      return new TextDecoder("UTF-8");
    }
  }

  const textDecoder = get_text_decoder();

  const js_string_from_raw = function (mu8, ptr, len) {
    const u8s = mu8.subarray(ptr, ptr + len);
    return textDecoder.decode(u8s);
  };

  function get_text_encoder() {
    if (typeof TextEncoder == "function") {
      return new TextEncoder("UTF-8");
    } else {
      const util = require("util");
      const TextEncoder = util.TextEncoder;
      return new TextEncoder("UTF-8");
    }
  }

  const textEncoder = get_text_encoder();

  // this calls into wasm so invalidates existing memory.buffer
  // returns pointer and length wrapped in Uint32Array
  const new_pl_raw_string = function (wap_alloc, mem, js) {
    const u8s = textEncoder.encode(js);
    let pl = new Uint32Array(2);
    pl[1] = u8s.length;
    pl[0] = wap_alloc(pl[1]);
    const mu8 = new Uint8Array(mem.buffer);
    mu8.set(u8s, pl[0]);
    return pl;
  };

  const TYPE_NULL = 0;
  const TYPE_UNDEFINED = 1;
  const TYPE_BOOLEAN = 2;
  const TYPE_NUMBER = 3;
  const TYPE_STRING = 4;
  const TYPE_REF = 5;

  const get_args = function (mb, num_args, at_ptr, args_ptr) {
    const mu8 = new Uint8Array(mb);
    const mf64 = new Float64Array(mb);
    const mu32 = new Uint32Array(mb);

    let args = [];
    for (let i = 0; i < num_args; i++) {
      switch (mu8[at_ptr + i]) {
        case TYPE_NULL:
          args.push(null);
          break;
        case TYPE_UNDEFINED:
          args.push(undefined);
          break;
        case TYPE_BOOLEAN:
          args.push((mf64[args_ptr / 8 + i] === 0.0) ? false : true);
          break;
        case TYPE_NUMBER:
          args.push(mf64[args_ptr / 8 + i]);
          break;
        case TYPE_STRING:
          const s = js_string_from_raw(mu8, mu32[args_ptr / 4 + i], mu32[args_ptr / 4 + i + 1]);
          args.push(s);
          break;
        case TYPE_REF:
          args.push(wap.get(mf64[args_ptr / 8 + i]));
          break;
      }
    }
    return args;
  }


  const obj_call = function (obj, instance_handle, handle_of_function, num_args, at_ptr, args_ptr, ret_ptr) {
    const instance = wap.get(instance_handle);
    const the_function = wap.get(handle_of_function);
    const mem = instance.exports.memory;
    const args = get_args(mem.buffer, num_args, at_ptr, args_ptr);

    const ret = Reflect.apply(the_function, obj, args);

    if (ret === null) {
      debug("-> null");
      return TYPE_NULL;
    }
    const ret_type = typeof ret;
    if (ret_type === "undefined") {
      debug("-> undefined");
      return TYPE_UNDEFINED;

    } else if (ret_type === "boolean") {
      debug("-> boolean " + ret);
      const mf64 = new Float64Array(mem.buffer);
      mf64[ret_ptr / 8] = ret ? 1.0 : 0.0;
      return TYPE_BOOLEAN;

    } else if (ret_type === "number") {
      debug("-> number " + ret);
      const mf64 = new Float64Array(mem.buffer);
      mf64[ret_ptr / 8] = ret;
      return TYPE_NUMBER;

    } else if (ret_type === "string") {
      debug("-> string " + ret);
      const pl = new_pl_raw_string(instance.exports.wap_alloc, mem, ret);
      const mu32 = new Uint32Array(mem.buffer, ret_ptr, 2);
      mu32.set(pl);
      return TYPE_STRING;

    } else {
      const handle = new_handle();
      wap.set(handle, ret);
      debug("-> ref " + handle);
      const mf64 = new Float64Array(mem.buffer);
      mf64[ret_ptr / 8] = handle;
      return TYPE_REF;
    }
  };

  const WapImp = {
    get: function (instance_handle, from_handle, name_ptr, name_len, ret_ptr) {
      const instance = wap.get(instance_handle);
      const mem = instance.exports.memory;
      const mb = mem.buffer;
      const mu8 = new Uint8Array(mb);
      const name = js_string_from_raw(mu8, name_ptr, name_len);
      debug("i" + instance_handle + " get " + from_handle + "[" + name + "]");
      const from = wap.get(from_handle);

      const ret = from[name];

      if (ret === null) {
        debug("-> null");
        return TYPE_NULL;
      }
      const ret_type = typeof ret;
      if (ret_type === "undefined") {
        debug("-> undefined");
        return TYPE_UNDEFINED;

      } else if (ret_type === "boolean") {
        debug("-> boolean " + ret);
        const mf64 = new Float64Array(mb, ret_ptr, 1);
        mf64[0] = ret ? 1.0 : 0.0;
        return TYPE_BOOLEAN;

      } else if (ret_type === "number") {
        debug("-> number " + ret);
        const mf64 = new Float64Array(mb, ret_ptr, 1);
        mf64[0] = ret;
        return TYPE_NUMBER;

      } else if (ret_type === "string") {
        debug("-> string " + ret);
        const pl = new_pl_raw_string(instance.exports.wap_alloc, mem, ret);
        // note: mb and mu8 are now invalid, due to above call into wasm
        const mu32 = new Uint32Array(mem.buffer, ret_ptr, 2);
        mu32.set(pl);
        return TYPE_STRING;

      } else {
        const handle = new_handle();
        wap.set(handle, ret);
        debug("-> ref " + handle);
        const mf64 = new Float64Array(mb, ret_ptr, 1);
        mf64[0] = handle;
        return TYPE_REF;
      }
    },

    clone: function (from_handle) {
      const handle = new_handle();
      wap.set(handle, wap.get(from_handle));
      debug("clone " + from_handle + " to " + handle);
      return handle;
    },

    unmap: function (handle) {
      debug("will unmap: " + handle + " mapped total: " + wap.size);
      wap.delete(handle);
    },

    new_object: function () {
      const o = {};
      const handle = new_handle();
      wap.set(handle, o);
      debug("new object " + handle);
      return handle;
    },

    new_string: function (instance_handle, text_ptr, text_len) {
      const instance = wap.get(instance_handle);
      const mu8 = new Uint8Array(instance.exports.memory.buffer);
      const text = js_string_from_raw(mu8, text_ptr, text_len);
      const handle = new_handle();
      wap.set(handle, text);
      debug("i" + instance_handle + " new_string " + text + " " + handle);
      return handle;
    },

    new_construct: function (instance_handle, constructor_handle, num_args, at_ptr, args_ptr) {
      const instance = wap.get(instance_handle);
      const target = wap.get(constructor_handle);
      const args = get_args(instance.exports.memory.buffer, num_args, at_ptr, args_ptr);

      const c = Reflect.construct(target, args);

      const handle = new_handle();
      wap.set(handle, c);
      debug("i" + instance_handle + " new_construct " + text + " " + handle);
      return handle;
    },

    set_null: function (instance_handle, object_handle, name_ptr, name_len) {
      const instance = wap.get(instance_handle);
      const mu8 = new Uint8Array(instance.exports.memory.buffer);
      const o = wap.get(object_handle);
      const name = js_string_from_raw(mu8, name_ptr, name_len);
      o[name] = null;
    },

    set_undefined: function (instance_handle, object_handle, name_ptr, name_len) {
      const instance = wap.get(instance_handle);
      const mu8 = new Uint8Array(instance.exports.memory.buffer);
      const o = wap.get(object_handle);
      const name = js_string_from_raw(mu8, name_ptr, name_len);
      o[name] = undefined;
    },

    set_boolean: function (instance_handle, object_handle, name_ptr, name_len, val) {
      const instance = wap.get(instance_handle);
      const mu8 = new Uint8Array(instance.exports.memory.buffer);
      const o = wap.get(object_handle);
      const name = js_string_from_raw(mu8, name_ptr, name_len);
      o[name] = val > 0 ? true : false;
    },

    set_number: function (instance_handle, object_handle, name_ptr, name_len, val) {
      const instance = wap.get(instance_handle);
      const mu8 = new Uint8Array(instance.exports.memory.buffer);
      const o = wap.get(object_handle);
      const name = js_string_from_raw(mu8, name_ptr, name_len);
      o[name] = val;
    },

    set_string: function (instance_handle, object_handle, name_ptr, name_len, val_ptr, val_len) {
      const instance = wap.get(instance_handle);
      const mu8 = new Uint8Array(instance.exports.memory.buffer);
      const o = wap.get(object_handle);
      const name = js_string_from_raw(mu8, name_ptr, name_len);
      const val = js_string_from_raw(mu8, val_ptr, val_len);
      o[name] = val;
    },

    set_ref: function (instance_handle, object_handle, name_ptr, name_len, handle) {
      const instance = wap.get(instance_handle);
      const mu8 = new Uint8Array(instance.exports.memory.buffer);
      const o = wap.get(object_handle);
      const name = js_string_from_raw(mu8, name_ptr, name_len);
      o[name] = wap.get(handle);
    },

    call: function (instance_handle, handle_of_function, num_args, at_ptr, args_ptr, ret_ptr) {
      debug("i" + instance_handle + " call " + handle_of_function + "(" + num_args + " args)");
      return obj_call(this, instance_handle, handle_of_function, num_args, at_ptr, args_ptr, ret_ptr);
    },

    bound_call: function (instance_handle, handle_of_object, handle_of_function, num_args, at_ptr, args_ptr, ret_ptr) {
      debug("i" + instance_handle + " call " + handle_of_object + "." + handle_of_function + "(" + num_args + " args)");
      const obj = wap.get(handle_of_object);
      return obj_call(obj, instance_handle, handle_of_function, num_args, at_ptr, args_ptr, ret_ptr);
    },

    instanceof: function (instance_handle, handle_of_object, constructor_handle) {
      const instance = wap.get(instance_handle);
      const mu8 = new Uint8Array(instance.exports.memory.buffer);
      const obj = wap.get(handle_of_object);
      const type = wap.get(constructor_handle);
      debug("i" + instance_handle + " " + handle_of_object + ((obj instanceof type) ? " instance of " : " NOT instance of ") + constructor_handle);
      return obj instanceof type;
    },

    delete: function (instance_handle, handle_of_object, name_ptr, name_len) {
      const instance = wap.get(instance_handle);
      const mu8 = new Uint8Array(instance.exports.memory.buffer);
      const obj = wap.get(handle_of_object);
      const name = js_string_from_raw(mu8, name_ptr, name_len);
      delete obj[name];
    },

    eq: function (first_handle, second_handle) {
      const first = wap.get(first_handle);
      const second = wap.get(second_handle);
      return first === second;
    },
  };

  let lib = {};
  lib.wap = function (wasm_url, imports) {
    const out = {};
    out.status = "starting";

    if (typeof imports === "undefined") {
      imports = {};
    }
    if (typeof imports.env === "undefined") {
      imports.env = {};
    }

    //https://github.com/rust-lang/rust/blob/560a5da9f1cc7f67d2fc372925aef18c96c82629/src/libstd/sys/wasm/cmath.rs#L77-L119
    imports.env["Math_acos"] = Math.acos;
    imports.env["Math_asin"] = Math.asin;
    imports.env["Math_atan"] = Math.atan;
    imports.env["Math_atan2"] = Math.atan2;
    imports.env["Math_cbrt"] = Math.cbrt;
    imports.env["Math_cosh"] = Math.cosh;
    imports.env["Math_expm1"] = Math.expm1;
    imports.env["fdim"] = function (a, b) { return Math.max(a - b, 0.0); };
    imports.env["Math_log1p"] = Math.log1p;
    imports.env["Math_sinh"] = Math.sinh;
    imports.env["Math_tan"] = Math.tan;
    imports.env["Math_tanh"] = Math.tanh;
    imports.env["Math_hypot"] = Math.hypot;

    imports.env["fmod"] = function (a, b) { return a % b; };
    imports.env["cosf"] = Math.cos;
    imports.env["cos"] = Math.cos;
    imports.env["sinf"] = Math.sin;
    imports.env["sin"] = Math.sin;

    imports["WapImp"] = WapImp;

    let ab = undefined;
    if (typeof fetch === "function") {
      ab = fetch(wasm_url)
        .then(response => response.arrayBuffer());
    } else {
      const fs = require('fs');
      const util = require('util');
      const readFile = util.promisify(fs.readFile);
      ab = readFile(wasm_url);
    }
    ab.then(bytes => WebAssembly.instantiate(bytes, imports))
      .then(({ module, instance }) => {
        out.module = module;
        out.instance = instance;
        const inst = new_handle();
        wap.set(inst, instance);
        debug("instance " + inst);
        const glob = new_handle();
        wap.set(glob, g());
        debug("global " + glob);
        out.status = "pre begin";
        instance.exports.wap_begin(inst, glob);
        out.status = "begun";
      })
      .catch((reason) => {
        out.status = "error";
        console.log("promise err cought:");
        console.log(reason);
        if (typeof window == "object") {
          window.addEventListener("DOMContentLoaded", () =>
            window.document.body.innerHTML = "Script error. Check console for detail.");
        }
      });

    return out;
  };

  const g = function () {
    if (typeof window === "object")
      return window;
    else if (typeof self === "object")
      return self;
    else if (typeof global === "object")
      return global;
    else
      return this;
  };

  //todo make as real lib like Math
  g().Wap = Object.seal(lib);

  // simple node.js export
  // todo try Worker support
  if (typeof global === "object" && typeof exports === "object") {
    exports.wap = lib.wap;
  }

}
