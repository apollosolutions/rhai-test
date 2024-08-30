mod engine;
mod expector;
mod extensions;
mod test_container;
mod test_runner;

use clap::Parser;
use expector::Expector;
use glob::glob;
use rhai::module_resolvers::FileModuleResolver;
use rhai::{Dynamic, Engine, FnPtr, AST};
use serde::Deserialize;
use std::fs::{self};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use test_container::TestContainer;
use test_runner::TestRunner;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "rhai-test.config.json")]
    config: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    #[serde(rename = "testMatch")]
    test_match: Vec<String>,
}

fn main() {
    let start_time = Instant::now();

    let args = Args::parse();
    let config_string = fs::read_to_string(args.config).expect("Unable to read config file");

    let config: Config = serde_json::from_str(&config_string).expect("JSON was not well-formatted");

    let mut test_files: Vec<String> = Vec::new();

    for path in &config.test_match {
        for entry in glob(path).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => test_files.push(path.display().to_string()),
                Err(e) => println!("{:?}", e),
            }
        }
    }

    let test_container = Arc::new(Mutex::new(TestContainer::new()));
    let engine = Arc::new(Mutex::new(Engine::new()));
    let shared_ast: Arc<Mutex<Option<AST>>> = Arc::new(Mutex::new(None));

    let expectors = Arc::new(Mutex::new(Vec::<Expector>::new()));
    let cloned_expectors = expectors.clone();
    let cloned_shared_ast = shared_ast.clone();

    {
        let mut engine_guard = engine.lock().unwrap();
        engine_guard
            .register_type_with_name::<Expector>("Expector")
            .register_fn("expect", move |value: Dynamic| {
                let mut expector = Expector::new(value);
                expector.attach_engine_and_ast(cloned_shared_ast.clone());
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

        let resolver = FileModuleResolver::new_with_path("examples"); // TODO: This should be configurable
        engine_guard.set_module_resolver(resolver);

        extensions::apollo::register_rhai_functions_and_types(&mut engine_guard);
        extensions::helpers::register_rhai_functions_and_types(&mut engine_guard);

        extensions::apollo::register_mocking_functions(&mut engine_guard);
    }

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

        let ast = {
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
                    Err(err) => {
                        println!("Eval error: {}", err);
                    }
                }
            }
            Err(error) => {
                println!("Compilation Error: {}", error);
            }
        }
    }
    let end_time = Instant::now();

    test_container.lock().unwrap().print_results();

    let elapsed_time = end_time - start_time;

    let time_string = if elapsed_time.as_secs_f64() < 1.0 {
        format!("{:.2} ms", elapsed_time.as_secs_f64() * 1000.0)
    } else {
        format!("{:.2} s", elapsed_time.as_secs_f64())
    };

    println!("Time:        {}", time_string)
}
