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

const js_string_from_rust_raw = function (memory, ptr) {
  let end = ptr;
  while (memory[end] !== 0) {
    end++;
  }
  const u8 = memory.subarray(ptr, end);
  const td = new TextDecoder("UTF-8");
  return td.decode(u8);
};

const new_rust_raw_string = function (wap_alloc, memory, js) {
  const te = new TextEncoder("UTF-8");
  const u8 = te.encode(js);
  const len = u8.length;
  const ptr = wap_alloc(len + 1);
  memory.set(u8, ptr);
  memory[ptr + len] = 0;
  return ptr;
};

const wap_ret_helper = function (wap_alloc, memory, ret, ret_ptr) {
  const ret_type = typeof ret;
  if (ret === null) {
    memory[ret_ptr] = 0;
    debug("-> null");

  } else if (ret_type === "undefined") {
    memory[ret_ptr] = 1;
    debug("-> undefined");

  } else if (ret_type === "boolean") {
    memory[ret_ptr] = 2;
    debug("-> boolean " + ret);
    memory[ret_ptr + 1] = ret ? 1 : 0;

  } else if (ret_type === "number") {
    memory[ret_ptr] = 3;
    debug("-> number " + ret);
    const b = new ArrayBuffer(8);
    const f64 = new Float64Array(b);
    f64[0] = ret;
    const u8 = new Uint8Array(b);
    memory.set(u8, ret_ptr + 1);

  } else if (ret_type === "string") {
    memory[ret_ptr] = 4;
    debug("-> string " + ret);
    const ptr = new_rust_raw_string(wap_alloc, memory, ret);
    const b = new ArrayBuffer(4);
    const u32 = new Uint32Array(b);
    u32[0] = ptr;
    const u8 = new Uint8Array(b);
    memory.set(u8, ret_ptr + 1);

  } else {
    memory[ret_ptr] = 5;
    const index = new_key();
    debug("-> ref " + index);
    wap.set(index, ret);
    const b = new ArrayBuffer(8);
    const f64 = new Float64Array(b);
    f64[0] = index;
    const u8 = new Uint8Array(b);
    memory.set(u8, ret_ptr + 1);
  }
  //  debug("mem" + memory[ret_ptr] + memory[ret_ptr+1] + memory[ret_ptr+2] +
  //                                        memory[ret_ptr+3] + memory[ret_ptr+4] +
  //                                        memory[ret_ptr+5] + memory[ret_ptr+6] +
  //                                        memory[ret_ptr+7] + memory[ret_ptr+8])
}

const wap_get = function (instance_index, from_index, name_ptr, ret_ptr) {
  const instance = wap.get(instance_index);
  const memory = new Uint8Array(instance.exports.memory.buffer);
  const name = js_string_from_rust_raw(memory, name_ptr);
  debug("i" + instance_index + " get " + from_index + "[" + name + "]");
  const from = wap.get(from_index);

  const ret = from[name];

  wap_ret_helper(instance.exports.wap_alloc, memory, ret, ret_ptr);
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
  const memory = new Uint8Array(instance.exports.memory.buffer);
  const text = js_string_from_rust_raw(memory, text_ptr);
  const index = new_key();
  wap.set(index, text);
  debug("i" + instance_index + " new_string " + text + " " + index);
  return index;
};

const set_null = function (instance_index, object_index, name_ptr) {
  const instance = wap.get(instance_index);
  const memory = new Uint8Array(instance.exports.memory.buffer);
  const o = wap.get(object_index);
  const name = js_string_from_rust_raw(memory, name_ptr);
  o[name] = null;
};

const set_undefined = function (instance_index, object_index, name_ptr) {
  const instance = wap.get(instance_index);
  const memory = new Uint8Array(instance.exports.memory.buffer);
  const o = wap.get(object_index);
  const name = js_string_from_rust_raw(memory, name_ptr);
  o[name] = undefined;
};

const set_boolean = function (instance_index, object_index, name_ptr, val) {
  const instance = wap.get(instance_index);
  const memory = new Uint8Array(instance.exports.memory.buffer);
  const o = wap.get(object_index);
  const name = js_string_from_rust_raw(memory, name_ptr);
  o[name] = val > 0 ? true : false;
};

const set_number = function (instance_index, object_index, name_ptr, val) {
  const instance = wap.get(instance_index);
  const memory = new Uint8Array(instance.exports.memory.buffer);
  const o = wap.get(object_index);
  const name = js_string_from_rust_raw(memory, name_ptr);
  o[name] = val;
};

const set_string = function (instance_index, object_index, name_ptr, ptr) {
  const instance = wap.get(instance_index);
  const memory = new Uint8Array(instance.exports.memory.buffer);
  const o = wap.get(object_index);
  const name = js_string_from_rust_raw(memory, name_ptr);
  const val = js_string_from_rust_raw(memory, ptr);
  o[name] = val;
};

const set_ref = function (instance_index, object_index, name_ptr, index) {
  const instance = wap.get(instance_index);
  const memory = new Uint8Array(instance.exports.memory.buffer);
  const o = wap.get(object_index);
  const name = js_string_from_rust_raw(memory, name_ptr);
  o[name] = wap.get(index);
};

const obj_call = function (obj, instance_index, index_of_function, num_args, args_ptr, ret_ptr) {
  const instance = wap.get(instance_index);
  const the_function = wap.get(index_of_function);

  const memory = new Uint8Array(instance.exports.memory.buffer);
  let args = [];
  for (let i = 0; i < num_args; i++) {
    switch (memory[args_ptr + i * 9]) {
      case 0:
        args.push(null);
        break;
      case 1:
        args.push(undefined);
        break;
      case 2:
        args.push((memory[args_ptr + i * 9 + 1] > 0) ? true : false);
        break;
      case 3:
        {
          const b = new ArrayBuffer(8);
          const u8 = new Uint8Array(b);
          u8.set(memory.subarray(args_ptr + + i * 9 + 1, args_ptr + + i * 9 + 9));
          const f64 = new Float64Array(b);
          args.push(f64[0]);
        }
        break;
      case 4:
        {
          const b = new ArrayBuffer(4);
          const u8 = new Uint8Array(b);
          u8.set(memory.subarray(args_ptr + i * 9 + 1, args_ptr + i * 9 + 5));
          const u32 = new Uint32Array(b);
          const s = js_string_from_rust_raw(memory, u32[0]);
          args.push(s);
        }
        break;
      case 5:
        {
          const b = new ArrayBuffer(8);
          const u8 = new Uint8Array(b);
          u8.set(memory.subarray(args_ptr + + i * 9 + 1, args_ptr + + i * 9 + 9));
          const f64 = new Float64Array(b);
          args.push(wap.get(f64[0]));
        }
        break;
    }
  }

  const ret = the_function.apply(obj, args);

  wap_ret_helper(instance.exports.wap_alloc, memory, ret, ret_ptr);
}

const call = function (instance_index, index_of_function, num_args, args_ptr, ret_ptr) {
  debug("i" + instance_index + " call " + index_of_function + "(" + num_args + " args)");
  return obj_call(this, instance_index, index_of_function, num_args, args_ptr, ret_ptr);
};

const bound_call = function (instance_index, index_of_object, index_of_function, num_args, args_ptr, ret_ptr) {
  debug("i" + instance_index + " call " + index_of_object + "." + index_of_function + "(" + num_args + " args)");
  const obj = wap.get(index_of_object);
  return obj_call(obj, instance_index, index_of_function, num_args, args_ptr, ret_ptr);
};

const wap_instanceof = function (instance_index, index_of_object, of_ptr) {
  const instance = wap.get(instance_index);
  const memory = new Uint8Array(instance.exports.memory.buffer);
  const obj = wap.get(index_of_object);
  const type = js_string_from_rust_raw(memory, of_ptr);
  debug("i" + instance_index + " " + index_of_object + (eval("obj instanceof " + type) ? " instance of " : " NOT instance of ") + type);
  return eval("obj instanceof " + type);
}

const wap_delete = function (instance_index, index_of_object, name_ptr) {
  const instance = wap.get(instance_index);
  const memory = new Uint8Array(instance.exports.memory.buffer);
  const obj = wap.get(index_of_object);
  const name = js_string_from_rust_raw(memory, name_ptr);
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
