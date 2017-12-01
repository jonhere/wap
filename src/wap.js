// file copyright release to public domain

const debug = function (s) {
  console.log(s);
};

const wap = new Map();

let next = 1; // could start negative to double max range but small good to debug
const new_key = function () {
  // breaks at f64 at 2^53
  return next++;
}

const js_string_from_rust_raw = function (mu8, ptr) {
  let end = ptr;
  while (mu8[end] !== 0) {
    end++;
  }
  const u8s = mu8.subarray(ptr, end);
  const td = new TextDecoder("UTF-8");
  return td.decode(u8s);
};

const new_rust_raw_string = function (wap_alloc, mu8, js) {
  const te = new TextEncoder("UTF-8");
  const u8s = te.encode(js);
  const len = u8s.length;
  const ptr = wap_alloc(len + 1);
  mu8.set(u8s, ptr);
  mu8[ptr + len] = 0;
  return ptr;
};

const TYPE_NULL = 0;
const TYPE_UNDEFINED = 1;
const TYPE_BOOLEAN = 2;
const TYPE_NUMBER = 3;
const TYPE_STRING = 4;
const TYPE_REF = 5;

const wap_get = function (instance_index, from_index, name_ptr, ret_ptr) {
  const instance = wap.get(instance_index);
  const mb = instance.exports.memory.buffer;
  const mu8 = new Uint8Array(mb);
  const name = js_string_from_rust_raw(mu8, name_ptr);
  debug("i" + instance_index + " get " + from_index + "[" + name + "]");
  const from = wap.get(from_index);

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
    const ptr = new_rust_raw_string(instance.exports.wap_alloc, mu8, ret);
    const mu32 = new Uint32Array(mb, ret_ptr, 1);
    mu32[0] = ptr;
    return TYPE_STRING;

  } else {
    const index = new_key();
    wap.set(index, ret);
    debug("-> ref " + index);
    const mf64 = new Float64Array(mb, ret_ptr, 1);
    mf64[0] = index;
    return TYPE_REF;
  }
};

const clone = function (from_index) {
  const index = new_key();
  wap.set(index, wap.get(from_index));
  debug("clone " + from_index + " to " + index);
  return index;
}

const unmap = function (index) {
  debug("will unmap: " + index + " mapped total: " + wap.size);
  wap.delete(index);
};

const new_object = function () {
  const o = {};
  const index = new_key();
  wap.set(index, o);
  debug("new object " + index);
  return index;
};

const new_string = function (instance_index, text_ptr) {
  const instance = wap.get(instance_index);
  const mu8 = new Uint8Array(instance.exports.memory.buffer);
  const text = js_string_from_rust_raw(mu8, text_ptr);
  const index = new_key();
  wap.set(index, text);
  debug("i" + instance_index + " new_string " + text + " " + index);
  return index;
};

const set_null = function (instance_index, object_index, name_ptr) {
  const instance = wap.get(instance_index);
  const mu8 = new Uint8Array(instance.exports.memory.buffer);
  const o = wap.get(object_index);
  const name = js_string_from_rust_raw(mu8, name_ptr);
  o[name] = null;
};

const set_undefined = function (instance_index, object_index, name_ptr) {
  const instance = wap.get(instance_index);
  const mu8 = new Uint8Array(instance.exports.memory.buffer);
  const o = wap.get(object_index);
  const name = js_string_from_rust_raw(mu8, name_ptr);
  o[name] = undefined;
};

const set_boolean = function (instance_index, object_index, name_ptr, val) {
  const instance = wap.get(instance_index);
  const mu8 = new Uint8Array(instance.exports.memory.buffer);
  const o = wap.get(object_index);
  const name = js_string_from_rust_raw(mu8, name_ptr);
  o[name] = val > 0 ? true : false;
};

const set_number = function (instance_index, object_index, name_ptr, val) {
  const instance = wap.get(instance_index);
  const mu8 = new Uint8Array(instance.exports.memory.buffer);
  const o = wap.get(object_index);
  const name = js_string_from_rust_raw(mu8, name_ptr);
  o[name] = val;
};

const set_string = function (instance_index, object_index, name_ptr, ptr) {
  const instance = wap.get(instance_index);
  const mu8 = new Uint8Array(instance.exports.memory.buffer);
  const o = wap.get(object_index);
  const name = js_string_from_rust_raw(mu8, name_ptr);
  const val = js_string_from_rust_raw(mu8, ptr);
  o[name] = val;
};

const set_ref = function (instance_index, object_index, name_ptr, index) {
  const instance = wap.get(instance_index);
  const mu8 = new Uint8Array(instance.exports.memory.buffer);
  const o = wap.get(object_index);
  const name = js_string_from_rust_raw(mu8, name_ptr);
  o[name] = wap.get(index);
};

const obj_call = function (obj, instance_index, index_of_function, num_args, at_ptr, args_ptr, ret_ptr) {
  const instance = wap.get(instance_index);
  const mb = instance.exports.memory.buffer;
  const mu8 = new Uint8Array(mb);
  const mf64 = new Float64Array(mb);
  const mu32 = new Uint32Array(mb);
  const the_function = wap.get(index_of_function);

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
        const s = js_string_from_rust_raw(mu8, mu32[args_ptr / 4 + i]);
        args.push(s);
        break;
      case TYPE_REF:
        args.push(wap.get(mf64[args_ptr / 8 + i]));
        break;
    }
  }

  const ret = the_function.apply(obj, args);

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
    mf64[ret_ptr / 8] = ret ? 1.0 : 0.0;
    return TYPE_BOOLEAN;

  } else if (ret_type === "number") {
    debug("-> number " + ret);
    mf64[ret_ptr / 8] = ret;
    return TYPE_NUMBER;

  } else if (ret_type === "string") {
    debug("-> string " + ret);
    const ptr = new_rust_raw_string(instance.exports.wap_alloc, mu8, ret);
    mu32[ret_ptr / 4] = ptr;
    return TYPE_STRING;

  } else {
    const index = new_key();
    wap.set(index, ret);
    debug("-> ref " + index);
    mf64[ret_ptr / 8] = index;
    return TYPE_REF;
  }
}

const call = function (instance_index, index_of_function, num_args, at_ptr, args_ptr, ret_ptr) {
  debug("i" + instance_index + " call " + index_of_function + "(" + num_args + " args)");
  return obj_call(this, instance_index, index_of_function, num_args, at_ptr, args_ptr, ret_ptr);
};

const bound_call = function (instance_index, index_of_object, index_of_function, num_args, at_ptr, args_ptr, ret_ptr) {
  debug("i" + instance_index + " call " + index_of_object + "." + index_of_function + "(" + num_args + " args)");
  const obj = wap.get(index_of_object);
  return obj_call(obj, instance_index, index_of_function, num_args, at_ptr, args_ptr, ret_ptr);
};

const wap_instanceof = function (instance_index, index_of_object, of_ptr) {
  const instance = wap.get(instance_index);
  const mu8 = new Uint8Array(instance.exports.memory.buffer);
  const obj = wap.get(index_of_object);
  const type = js_string_from_rust_raw(mu8, of_ptr);
  debug("i" + instance_index + " " + index_of_object + (eval("obj instanceof " + type) ? " instance of " : " NOT instance of ") + type);
  return eval("obj instanceof " + type);
}

const wap_delete = function (instance_index, index_of_object, name_ptr) {
  const instance = wap.get(instance_index);
  const mu8 = new Uint8Array(instance.exports.memory.buffer);
  const obj = wap.get(index_of_object);
  const name = js_string_from_rust_raw(mu8, name_ptr);
  delete obj[name];
}

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

  imports.env["wap_get"] = wap_get;
  imports.env["wap_clone"] = clone;
  imports.env["wap_unmap"] = unmap;
  imports.env["wap_new_object"] = new_object;
  imports.env["wap_new_string"] = new_string;
  imports.env["wap_set_null"] = set_null;
  imports.env["wap_set_undefined"] = set_undefined;
  imports.env["wap_set_boolean"] = set_boolean;
  imports.env["wap_set_number"] = set_number;
  imports.env["wap_set_string"] = set_string;
  imports.env["wap_set_ref"] = set_ref;
  imports.env["wap_call"] = call;
  imports.env["wap_bound_call"] = bound_call;
  imports.env["wap_instanceof"] = wap_instanceof;
  imports.env["wap_delete"] = wap_delete;


  fetch(wasm_url)
    .then(response => response.arrayBuffer())
    .then(bytes => WebAssembly.instantiate(bytes, imports))
    .then(({ module, instance }) => {
      out.module = module;
      out.instance = instance;
      const inst = new_key();
      wap.set(inst, instance);
      debug("instance " + inst);
      const glob = new_key();
      wap.set(glob, g());
      debug("global (window) " + glob);
      out.status = "pre begin";
      instance.exports.wap_begin(inst, glob);
      out.status = "begun";
    })
    .catch(function (reason) {
      out.status = "error";
      debug("promise err cought");
      debug(reason);
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
