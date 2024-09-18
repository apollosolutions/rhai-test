use crate::coverage_reporting::test_coverage_container::TestCoverageContainer;
use rhai::Engine;
use std::sync::{Arc, Mutex};

/// Registers all the instrumentation functions for test coverage
pub fn register_rhai_functions_and_types(
    engine: &mut Engine,
    test_coverage_container: Arc<Mutex<TestCoverageContainer>>,
) {
    let test_coverage_container_functions_clone = test_coverage_container.clone();
    let rhai_test_coverage_instrument_function =
        move |function_name: String, source: String, line_number: i64| {
            test_coverage_container_functions_clone
                .lock()
                .unwrap()
                .function_called(function_name, source, line_number);
        };

    let test_coverage_container_statements_clone = test_coverage_container.clone();
    let rhai_test_coverage_instrument_statement = move |source: String, line_number: i64| {
        test_coverage_container_statements_clone
            .lock()
            .unwrap()
            .statement_called(source, line_number);
    };

    let test_coverage_container_branches_clone = test_coverage_container.clone();
    let rhai_test_coverage_instrument_branch = move |source: String, line_number: i64| {
        test_coverage_container_branches_clone
            .lock()
            .unwrap()
            .branch_called(source, line_number);
    };

    engine.register_fn(
        "rhai_test_coverage_instrument_function",
        rhai_test_coverage_instrument_function,
    );

    engine.register_fn(
        "rhai_test_coverage_instrument_statement",
        rhai_test_coverage_instrument_statement,
    );

    engine.register_fn(
        "rhai_test_coverage_instrument_branch",
        rhai_test_coverage_instrument_branch,
    );
}
