use crate::extensions;
use rhai::{module_resolvers::FileModuleResolver, Engine};

pub fn create_engine() -> Engine {
    let mut engine = Engine::new();
    let resolver = FileModuleResolver::new_with_path("examples"); // TODO: This should be configurable
    engine.set_module_resolver(resolver);

    extensions::apollo::register_rhai_functions_and_types(&mut engine);
    extensions::helpers::register_rhai_functions_and_types(&mut engine);
    extensions::apollo::register_mocking_functions(&mut engine);

    engine
}
