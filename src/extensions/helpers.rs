use rhai::Engine;
use std::env;

pub fn register_rhai_functions_and_types(engine: &mut Engine) {
    engine
        .register_type_with_name::<TestHelpers>("TestHelpers")
        .register_fn("get_testing_utils", TestHelpers::new)
        .register_fn("set_env", TestHelpers::set_env);
}

#[derive(Debug, Clone)]
pub struct TestHelpers {}

impl TestHelpers {
    pub fn new() -> Self {
        Self {}
    }

    pub fn set_env(&mut self, name: &str, value: &str) {
        env::set_var(name, value);
    }
}
