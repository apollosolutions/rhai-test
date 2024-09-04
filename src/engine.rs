use std::sync::{Arc, Mutex};

use crate::{
    extensions::{
        self,
        file_coverage::{FileCoverageModuleResolver, TestCoverageContainer},
    },
    Config,
};
use rhai::{module_resolvers::FileModuleResolver, Engine};

pub fn create_engine(
    test_coverage_container: Arc<Mutex<TestCoverageContainer>>,
    config: Arc<Mutex<Config>>,
) -> Engine {
    let mut engine = Engine::new();

    if (config.lock().unwrap().coverage.unwrap_or_default()) {
        let resolver = FileCoverageModuleResolver::new("examples", test_coverage_container.clone());
        engine.set_module_resolver(resolver);
    } else {
        let resolver = FileModuleResolver::new_with_path("examples"); // TODO: This should be configurable
        engine.set_module_resolver(resolver);
    }

    extensions::apollo::register_rhai_functions_and_types(&mut engine);
    extensions::helpers::register_rhai_functions_and_types(&mut engine);
    extensions::apollo::register_mocking_functions(&mut engine);
    extensions::file_coverage::register_rhai_functions_and_types(
        &mut engine,
        test_coverage_container,
    );

    engine
}
