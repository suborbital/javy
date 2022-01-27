mod engine;
mod js_binding;
mod serialize;
mod transcode;

use js_binding::{context::Context, value::Value};

use once_cell::sync::OnceCell;
use std::io::{self, Read};

use quickjs_sys::{JSContext, JSValue};
use std::mem;
use std::os::raw::c_int;
use std::slice;

use convert_case::{Case, Casing};

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut JS_CONTEXT: OnceCell<Context> = OnceCell::new();
static mut ENTRYPOINT: (OnceCell<Value>, OnceCell<Value>) = (OnceCell::new(), OnceCell::new());
static SCRIPT_NAME: &str = "script.js";

// Wraps the call to the host function by either casting the result to a
// JSValue or returning `undefined` if the host function doesn't return a value
// (as _all_ JS functions must return a value). This is really only meant to be
// used by the `bind_imports!` macro to keep it relatively simple.
//
//  bind_js_return!(context, host_func(arg), i32)
//      =>
//  host_func(arg) as JSValue
//
//  bind_js_return!(context, host_func(arg),)
//      =>
//  {
//      host_func(arg);
//      context.undefined_value().unwrap().as_raw()
//  }
macro_rules! bind_js_return {
    ($context:ident, $expr:expr, $return_ty:ty) => {
        $expr as JSValue
    };
    ($context:ident, $expr:expr,) => {{
        $expr;
        $context.undefined_value().unwrap().as_raw()
    }};
}

// TODO: Support i64, f32, and f64 types.
// Creates JS closures for all host functions and assigns them to an import
// object.
//
//  bind_imports!(context, import_obj, {
//      fn get_ffi_result(pointer: *const u8, ident: i32) -> i32;
//      fn return_result(result_pointer: *const u8, result_size: i32, ident: i32);
//  })
//
// The output of this macro is somewhat similar to the `build_realloc`
// function, except that it assigns the closure value to the import object.
macro_rules! bind_imports {
    ($context:ident, $import_obj:ident, {$(fn $name:ident ($($param:ident : $param_ty:ty),*$(,)?) $(-> $return_ty:ty)?);*;}) => {{
        $({
            let callback = |_ctx: *mut JSContext, _this: JSValue, argc: c_int, argv: *mut JSValue, _magic: c_int| {
                extern "C" {
                    fn $name($($param : $param_ty),*) $(-> $return_ty)?;
                }
                let args = slice::from_raw_parts(argv, argc as usize);
                if let [$($param),*] = *args {
                    bind_js_return!($context, $name($($param as $param_ty),*), $($return_ty)?)
                } else {
                    panic!("Incorrect number of arguments")
                }
            };
            $import_obj
                .set_property(
                    stringify!($name).to_case(Case::Kebab),
                    $context.new_callback(callback).unwrap(),
                )
                .expect(stringify!(failed to set property $name));
        });*
    }};
}

// Provides the JS a way to allocate memory blocks to pass raw bytes to the host
fn build_realloc(context: &Context) -> Value {
    unsafe {
        context
            .new_callback(
                |_ctx: *mut JSContext,
                 _this: JSValue,
                 argc: c_int,
                 argv: *mut JSValue,
                 _magic: c_int| {
                    let args = slice::from_raw_parts(argv, argc as usize);

                    let ptr = args[0];
                    let orig_size = args[1];
                    let _align = args[2];
                    let new_size = args[3];

                    let old_mem = slice::from_raw_parts(ptr as *const u8, orig_size as usize);

                    let mut buffer = Vec::with_capacity(new_size as usize);
                    buffer.extend_from_slice(old_mem);

                    let pointer = buffer.as_mut_ptr();

                    mem::forget(buffer);

                    pointer as JSValue
                },
            )
            .unwrap()
    }
}

// Provides a buffer to the full JS memory that refreshes as memory grows
fn build_memory(context: &Context) -> Value {
    // This memory object pretends to be a WebAssembly.Memory object.
    // The bindings only expect the `buffer` property to be available,
    // so we provide an ArrayBuffer backed by the full module's memory.
    // It's important to note that this memory is shared between both
    // the JS engine _and_ the Rust code, so great care should be taken
    // to avoid memory corruption from the JS side.
    let memory = context.object_value().unwrap();
    // TODO: Provide this ArrayBuffer as a getter that returns an ArrayBuffer
    // for the full memory as it grows. If the JS attemtps to read or write to
    // a region that hasn't yet been claimed by the WebAssembly module, it will
    // trap; granted, it would have caused an error anyway. The difference is
    // that the trap cannot be caught. This generally shouldn't be an issue, as
    // all interactions with memory are done by generated code.
    memory
        .set_property("buffer", context.memory_value().unwrap())
        .expect("failed to set buffer on memory");
    memory
}

fn setup_imports(context: &Context, import_obj: &Value) {
    // Create JS closures for all host functions.
    unsafe {
        bind_imports!(context, import_obj, {
            fn log_msg(pointer: *const u8, result_size: i32, level: i32, ident: i32);
            fn fetch_url(
                method: i32,
                url_pointer: *const u8,
                url_size: i32,
                body_pointer: *const u8,
                body_size: i32,
                ident: i32,
            ) -> i32;
            fn graphql_query(
                endpoint_pointer: *const u8,
                endpoint_size: i32,
                query_pointer: *const u8,
                query_size: i32,
                ident: i32,
            ) -> i32;
            fn cache_set(
                key_pointer: *const u8,
                key_size: i32,
                value_pointer: *const u8,
                value_size: i32,
                ttl: i32,
                ident: i32,
            ) -> i32;
            fn cache_get(key_pointer: *const u8, key_size: i32, ident: i32) -> i32;
            fn request_get_field(
                field_type: i32,
                key_pointer: *const u8,
                key_size: i32,
                ident: i32,
            ) -> i32;
            fn request_set_field(
                field_type: i32,
                key_pointer: *const u8,
                key_size: i32,
                val_pointer: *const u8,
                val_size: i32,
                ident: i32,
            ) -> i32;
            fn get_ffi_result(pointer: *const u8, ident: i32) -> i32;
            fn return_result(result_pointer: *const u8, result_size: i32, ident: i32);
            fn return_error(code: i32, result_pointer: *const u8, result_size: i32, ident: i32);
        });
    }

    import_obj
        .set_property("canonical_abi_realloc", build_realloc(&context))
        .expect("failed to set realloc on import object");
    import_obj
        .set_property("memory", build_memory(&context))
        .expect("failed to set memory on import object");
}

// TODO
//
// AOT validations:
//  1. Ensure that the required exports are present
//  2. If not present just evaluate the top level statement (?)

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
        let mut context = Context::default();
        context.register_globals(io::stdout()).unwrap();

        let mut contents = String::new();
        io::stdin().read_to_string(&mut contents).unwrap();

        let _ = context.eval_global(SCRIPT_NAME, &contents).unwrap();
        let global = context.global_object().unwrap();
        let suborbital = global.get_property("Suborbital").unwrap();
        let main = suborbital.get_property("run_e").unwrap();
        let env = suborbital.get_property("env").unwrap();

        // Unlike the other language bindings, JS never considers itself a
        // WebAssembly module (which makes sense—JS is usually interoping with
        // wasm modules, not compiled to wasm itself). For this reason, the
        // bindings look at the host as though it's a wasm module. Even though
        // we're importing host functions, the JS bindings will consider them
        // exports from a wasm module. This doesn't have a major effect other
        // than some confusing terminology.
        let imports = context.object_value().unwrap();
        setup_imports(&context, &imports);

        // The JS expects our host functions under the `_exports` key on `env`.
        env.set_property("_exports", imports)
            .expect("failed to set _exports on env");

        JS_CONTEXT.set(context).unwrap();
        ENTRYPOINT.0.set(suborbital).unwrap();
        ENTRYPOINT.1.set(main).unwrap();
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn run_e(pointer: *mut u8, size: i32, ident: i32) {
    extern "C" {
        fn return_error(code: i32, result_pointer: *const u8, result_size: i32, ident: i32);
    }

    let in_bytes = Vec::from_raw_parts(pointer, size as usize, size as usize);

    let context = JS_CONTEXT.get().unwrap();
    let suborbital = ENTRYPOINT.0.get().unwrap();
    let main = ENTRYPOINT.1.get().unwrap();
    let input = context
        .value_from_bytes(in_bytes)
        .expect("Couldn't load input");
    let id = context.value_from_i32(ident).expect("Couldn't load ident");
    let output_value = main.call(&suborbital, &[input, id]);

    match output_value {
        Ok(_) => (),
        Err(err) => {
            let message = err.to_string();
            return_error(500, message.as_ptr(), message.len() as i32, ident);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn allocate(size: i32) -> *const u8 {
    let mut buffer = Vec::with_capacity(size as usize);

    let pointer = buffer.as_mut_ptr();

    mem::forget(buffer);

    pointer as *const u8
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn deallocate(pointer: *mut u8, size: i32) {
    drop(Vec::from_raw_parts(pointer, size as usize, size as usize))
}
