use colored::*;
use regex::Regex;
use rhai::{Engine, EvalAltResult, Module, ModuleResolver, Position, Scope};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tabled::{settings::Style, Table, Tabled};

use crate::coverage_reporting::test_coverage_container::TestCoverageContainer;

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

    let rhai_test_coverage_instrument_branch = move |source: String, line_number: i64| {
        /*println!(
            "rhai_test_coverage_instrument_statement: {} {}",
            source, line_number
        );*/
        /*test_coverage_container_clone
        .lock()
        .unwrap()
        .function_called(function_name, source, line_number);*/
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
