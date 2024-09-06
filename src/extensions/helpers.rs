use rhai::plugin::*;
use rhai::Engine;
use std::env;

pub fn register_rhai_functions_and_types(engine: &mut Engine) {
    let test_helpers_module = exported_module!(test_helpers);

    engine.register_static_module("test_helpers", test_helpers_module.into());
}

#[export_module]
mod test_helpers {

    #[rhai_fn()]
    pub(crate) fn set_env(name: &str, value: &str) {
        env::set_var(name, value);
    }
}
