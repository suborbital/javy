mod engine;
mod js_binding;
mod serialize;
mod transcode;

use js_binding::{context::Context, value::Value};

use once_cell::sync::OnceCell;
use std::io::{self, Read};
use suborbital::runnable::*;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut JS_CONTEXT: OnceCell<Context> = OnceCell::new();
static mut ENTRYPOINT: (OnceCell<Value>, OnceCell<Value>) = (OnceCell::new(), OnceCell::new());
static SCRIPT_NAME: &str = "script.js";

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
        let main = suborbital.get_property("main").unwrap();

        JS_CONTEXT.set(context).unwrap();
        ENTRYPOINT.0.set(suborbital).unwrap();
        ENTRYPOINT.1.set(main).unwrap();
    }
}

struct JsRunnable {}

impl Runnable for JsRunnable {
    fn run(&self, in_bytes: Vec<u8>) -> Result<Vec<u8>, RunErr> {
        unsafe {
            let context = JS_CONTEXT.get().unwrap();
            let suborbital = ENTRYPOINT.0.get().unwrap();
            let main = ENTRYPOINT.1.get().unwrap();
            let input = context
                .value_from_bytes(in_bytes)
                .expect("Couldn't load input");
            let output_value = main.call(&suborbital, &[input]);
            if output_value.is_err() {
                panic!("{}", output_value.unwrap_err().to_string());
            }
            match output_value {
                Ok(value) => {
                    let vec = context.value_to_bytes(value).unwrap();
                    Ok(vec)
                }
                Err(err) => {
                    let message = err.to_string();
                    Err(RunErr { code: 500, message })
                }
            }
        }
    }
}

// initialize the runner
static RUNNABLE: &JsRunnable = &JsRunnable {};

#[no_mangle]
pub extern "C" fn _start() {
    use_runnable(RUNNABLE);
}
