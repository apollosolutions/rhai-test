use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    coverage_reporting::{
        file_coverage_module_resolver::FileCoverageModuleResolver,
        test_coverage_container::TestCoverageContainer,
    },
    extensions::{self},
    Config,
};
use rhai::{module_resolvers::FileModuleResolver, Engine, Module};

pub fn create_engine(
    test_coverage_container: Arc<Mutex<TestCoverageContainer>>,
    config: Arc<Mutex<Config>>,
    module_cache: Arc<Mutex<BTreeMap<PathBuf, Arc<Module>>>>,
) -> Engine {
    let mut engine = Engine::new();
    let coverage = config.lock().unwrap().coverage;
    let base_path = config.lock().unwrap().base_path.clone();

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

    extensions::apollo::register_rhai_functions_and_types(&mut engine);
    extensions::helpers::register_rhai_functions_and_types(&mut engine);
    extensions::apollo::register_mocking_functions(&mut engine);
    extensions::file_coverage::register_rhai_functions_and_types(
        &mut engine,
        test_coverage_container,
    );

    engine
}
