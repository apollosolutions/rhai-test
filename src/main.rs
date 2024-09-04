mod coverage_reporting;
mod engine;
mod extensions;

use clap::Parser;
use colored::*;
use coverage_reporting::test_coverage_container::TestCoverageContainer;
use engine::engine::create_engine;
use engine::expector::Expector;
use engine::test_container::TestContainer;
use engine::test_runner::TestRunner;
use glob::glob;
use rhai::{Dynamic, FnPtr, AST};
use serde::Deserialize;
use std::fs::{self};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "rhai-test.config.json")]
    config: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(rename = "testMatch")]
    test_match: Vec<String>,

    coverage: Option<bool>,
}

fn main() {
    let start_time = Instant::now();

    let args = Args::parse();
    let config_string = fs::read_to_string(args.config).expect("Unable to read config file.");

    let config: Config =
        serde_json::from_str(&config_string).expect("Config file JSON was not well-formatted.");

    let mut test_files: Vec<String> = Vec::new();

    for path in &config.test_match {
        for entry in
            glob(path).expect("Failed to read a glob pattern for config file 'testMatch' value")
        {
            match entry {
                Ok(path) => test_files.push(path.display().to_string()),
                Err(e) => println!("{:?}", e),
            }
        }
    }

    let test_container = Arc::new(Mutex::new(TestContainer::new()));
    let test_coverage_container = Arc::new(Mutex::new(TestCoverageContainer::new()));
    let config_shared = Arc::new(Mutex::new(config));
    let engine = Arc::new(Mutex::new(create_engine(
        test_coverage_container.clone(),
        config_shared.clone(),
    )));
    let shared_ast: Arc<Mutex<Option<AST>>> = Arc::new(Mutex::new(None));

    let expectors = Arc::new(Mutex::new(Vec::<Expector>::new()));
    let cloned_expectors = expectors.clone();
    let cloned_shared_ast = shared_ast.clone();
    let test_coverage_container_clone = test_coverage_container.clone();
    let cloned_config_shared = config_shared.clone();

    // Attach the test specific functions to the engine
    {
        let mut engine_guard = engine.lock().unwrap();
        engine_guard
            .register_type_with_name::<Expector>("Expector")
            .register_fn("expect", move |value: Dynamic| {
                let mut expector = Expector::new(value);
                expector.attach(
                    cloned_shared_ast.clone(),
                    test_coverage_container_clone.clone(),
                    cloned_config_shared.clone(),
                );
                cloned_expectors.lock().unwrap().push(expector.clone());
                expector
            })
            .register_fn("not", Expector::not)
            .register_fn("to_be", Expector::to_be)
            .register_fn("to_match", Expector::to_match)
            .register_fn("to_throw", Expector::to_throw)
            .register_fn("to_throw_message", Expector::to_throw_message)
            .register_fn("to_throw_status", Expector::to_throw_status)
            .register_fn(
                "to_throw_status_and_message",
                Expector::to_throw_status_and_message,
            );
    }

    // Now run each test file
    for path in &test_files {
        let test_file_content = fs::read_to_string(path).expect("Unable to read rhai test file");

        let cloned_container = test_container.clone();
        let cloned_path = path.clone();

        let test = move |test_name: &str, func: FnPtr| {
            cloned_container
                .lock()
                .unwrap()
                .add_test(test_name, func, &cloned_path);
        };

        engine.lock().unwrap().register_fn("test", test);

        let ast: Result<AST, rhai::ParseError> = {
            let engine_guard = engine.lock().unwrap();
            engine_guard.compile(&test_file_content)
        };

        match ast {
            Ok(ast) => {
                {
                    let mut ast_lock = shared_ast.lock().unwrap();
                    *ast_lock = Some(ast.clone());
                }

                let eval_result = {
                    let engine_guard = engine.lock().unwrap();
                    engine_guard.eval::<()>(&test_file_content)
                };

                let ast_arc = Arc::new(Mutex::new(ast));

                match eval_result {
                    Ok(()) => {
                        let tests = {
                            let container = test_container.lock().unwrap();
                            container.get_tests().clone().to_vec()
                        };

                        let runner: TestRunner = TestRunner::new();
                        let run_result = runner.run_tests(
                            &engine.lock().unwrap(),
                            &ast_arc.lock().unwrap(),
                            &path,
                            &tests,
                        );

                        let mut container = test_container.lock().unwrap();
                        container.passed_tests += run_result.passed_tests;
                        container.failed_tests += run_result.failed_tests;
                        if run_result.failed_tests > 0 {
                            container.fail_suite(&path);
                        }
                    }
                    Err(error) => {
                        println!("{} {}", " FAIL ".white().on_red().bold(), path);
                        println!("\t{}", format!("Eval Error: {}", error).red());
                    }
                }
            }
            Err(error) => {
                println!("{} {}", " FAIL ".white().on_red().bold(), path);
                println!("\t{}", format!("Compilation Error: {}", error).red());
            }
        }
    }
    let end_time = Instant::now();

    if (config_shared.lock().unwrap().coverage.unwrap_or_default()) {
        test_coverage_container.lock().unwrap().print_results();
    }

    test_container.lock().unwrap().print_results();

    let elapsed_time = end_time - start_time;

    let time_string = if elapsed_time.as_secs_f64() < 1.0 {
        format!("{:.2} ms", elapsed_time.as_secs_f64() * 1000.0)
    } else {
        format!("{:.2} s", elapsed_time.as_secs_f64())
    };

    println!("Time:        {}", time_string)
}
