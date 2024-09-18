use super::logging_container::LoggingContainer;
use crate::{
    coverage_reporting::{
        file_coverage_module_resolver::FileCoverageModuleResolver,
        test_coverage_container::TestCoverageContainer,
    },
    extensions::{self},
    Config,
};
use rhai::{module_resolvers::FileModuleResolver, Engine, Module};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// Creates a rhai engine with all extensions attached to it
pub fn create_engine(
    test_coverage_container: Arc<Mutex<TestCoverageContainer>>,
    config: Arc<Mutex<Config>>,
    module_cache: Arc<Mutex<BTreeMap<PathBuf, Arc<Module>>>>,
    logging_container: Arc<Mutex<LoggingContainer>>,
) -> Engine {
    let mut engine = Engine::new();
    let coverage = config.lock().unwrap().coverage;
    let base_path = config.lock().unwrap().base_path.clone();

    // If we have opted into coverage reporting, we will use the special module loader, otherwise, use the default module loader
    if coverage.unwrap_or_default() {
        let resolver = FileCoverageModuleResolver::new(
            base_path,
            test_coverage_container.clone(),
            module_cache,
        );
        engine.set_module_resolver(resolver);
    } else {
        let resolver = FileModuleResolver::new_with_path(base_path);
        engine.set_module_resolver(resolver);
    }

    // Register all our functions and mocks
    extensions::apollo::register_rhai_functions_and_types(&mut engine, logging_container);
    extensions::helpers::register_rhai_functions_and_types(&mut engine);
    extensions::apollo::register_mocking_functions(&mut engine);
    extensions::file_coverage::register_rhai_functions_and_types(
        &mut engine,
        test_coverage_container,
    );

    engine
}
