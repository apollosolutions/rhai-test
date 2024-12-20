mod coverage_reporting;
mod engine;
mod extensions;
use clap::Parser;
use colored::*;
use coverage_reporting::test_coverage_container::TestCoverageContainer;
use engine::engine::create_engine;
use engine::error_handling::{get_stack_trace, get_stack_trace_output};
use engine::expector::Expector;
use engine::logging_container::LoggingContainer;
use engine::test_container::TestContainer;
use engine::test_runner::TestRunner;
use glob::glob;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use rhai::{Dynamic, FnPtr, Module, ParseError, AST};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "rhai-test.config.json")]
    config: String,

    #[arg(short, long, action)]
    watch: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(rename = "testMatch")]
    test_match: Vec<String>,

    #[serde(rename = "basePath")]
    base_path: String,

    coverage: Option<bool>,
}

fn main() {
    // Load config file based on arguments (or default)
    let args = Args::parse();
    let config_string = match fs::read_to_string(args.config.clone()) {
        Ok(file_content) => file_content,
        Err(error) => {
            let error_message = format!(
                "Configuration file not found at {}. Error: {}",
                args.config, error
            );
            println!("{}", error_message.red());
            exit(99);
        }
    };

    // Parse config file
    let config: Config = match serde_json::from_str(&config_string) {
        Ok(config_object) => config_object,
        Err(error) => {
            let error_message = format!(
                "Configuration file was not well-formatted JSON. Error: {}",
                error
            );
            println!("{}", error_message.red());
            exit(99);
        }
    };

    if args.watch {
        clear_screen();
        run_tests(config.clone(), true);
        if let Err(error) = watch(config) {
            let error_message = format!("Failure when watching files. Error: {}", error);
            println!("{}", error_message.red());
            exit(99);
        }
    } else {
        run_tests(config, false);
    }
}

/// Create a watcher at the configured file path and re-run the tests every time there is a file change detected
fn watch(config: Config) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    watcher.watch(Path::new(&config.base_path), RecursiveMode::Recursive)?;

    println!("Watching for changes...");

    for res in rx {
        match res {
            Ok(event) => {
                if matches!(
                    event.kind,
                    EventKind::Modify(notify::event::ModifyKind::Data(_))
                ) {
                    clear_screen();
                    run_tests(config.clone(), true);
                    println!("Watching for changes...");
                }
            }
            Err(error) => println!("Error: {error:?}"),
        }
    }

    Ok(())
}

/// Run the tests based on the provided config
fn run_tests(config: Config, is_watch_mode: bool) {
    let start_time = Instant::now();

    let mut test_files: Vec<String> = Vec::new();

    // Discover any test files that match the provided glob pattern(s)
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

    // Create all our core engine objects
    let test_container = Arc::new(Mutex::new(TestContainer::new()));
    let test_coverage_container = Arc::new(Mutex::new(TestCoverageContainer::new()));
    let config_shared = Arc::new(Mutex::new(config));
    let module_cache = Arc::new(Mutex::new(BTreeMap::<PathBuf, Arc<Module>>::new()));
    let logging_container = Arc::new(Mutex::new(LoggingContainer::new()));
    let engine = Arc::new(Mutex::new(create_engine(
        test_coverage_container.clone(),
        config_shared.clone(),
        module_cache.clone(),
        logging_container.clone(),
    )));
    let shared_ast: Arc<Mutex<Option<AST>>> = Arc::new(Mutex::new(None));

    // We're cloning stuff here so that it can be moved into the expect()` closure below
    let cloned_shared_ast = shared_ast.clone();
    let test_coverage_container_clone = test_coverage_container.clone();
    let cloned_config_shared = config_shared.clone();
    let cloned_module_cache = module_cache.clone();
    let cloned_logging_container = logging_container.clone();
    let cloned_container = test_container.clone();

    // Attach the test specific functions to the engine including defining our expect() function
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
                    cloned_module_cache.clone(),
                    cloned_logging_container.clone(),
                    cloned_container.clone(),
                );
                expector
            })
            .register_fn("not", Expector::not)
            .register_fn("to_be", Expector::to_be)
            .register_fn("to_exist", Expector::to_exist)
            .register_fn("to_match", Expector::to_match)
            .register_fn("to_throw", Expector::to_throw)
            .register_fn("to_throw_message", Expector::to_throw_message)
            .register_fn("to_throw_status", Expector::to_throw_status)
            .register_fn(
                "to_throw_status_and_message",
                Expector::to_throw_status_and_message,
            )
            .register_fn("to_log", Expector::to_log)
            .register_fn("to_log_message", Expector::to_log_message);
    }

    // Now parse and eval each test file
    for path in &test_files {
        let test_file_content = fs::read_to_string(path).expect("Unable to read rhai test file");

        // Clone these so they can be moved into the test() closure below
        let cloned_container = test_container.clone();
        let cloned_logging_container = logging_container.clone();
        let cloned_path = path.clone();

        // Add the test suite
        cloned_container.lock().unwrap().add_suite(&path);

        // Create and register our test() function
        let test = move |test_name: &str, func: FnPtr| {
            cloned_container
                .lock()
                .unwrap()
                .add_test(test_name, func, &cloned_path);
        };
        engine.lock().unwrap().register_fn("test", test);

        // Now we can evaluate our AST
        let ast: Result<AST, rhai::ParseError> = {
            let engine_guard = engine.lock().unwrap();
            engine_guard.compile(&test_file_content)
        };

        let cloned_container = test_container.clone();
        match ast {
            Ok(ast) => {
                // We're setting the AST into our shared_ast so that it can be used in our expect functions
                {
                    let mut ast_lock = shared_ast.lock().unwrap();
                    *ast_lock = Some(ast.clone());
                }

                // Now we can evaluate our test file
                let eval_result = {
                    let engine_guard = engine.lock().unwrap();
                    engine_guard.eval::<()>(&test_file_content)
                };
                let ast_arc = Arc::new(Mutex::new(ast));

                match eval_result {
                    Ok(()) => {
                        // Get the tests (note we're doing it in a let block so that the lock on the test_container only lasts for this block)
                        let tests = {
                            let container = test_container.lock().unwrap();
                            container.get_tests().clone().to_vec()
                        };

                        // Run the tests!
                        let runner: TestRunner = TestRunner::new();
                        let run_result = runner.run_tests(
                            &engine.lock().unwrap(),
                            &ast_arc.lock().unwrap(),
                            &path,
                            &tests,
                            cloned_logging_container.clone(),
                            cloned_container.clone(),
                        );

                        // Update our test container with what passed/failed
                        let mut container = test_container.lock().unwrap();
                        container.passed_tests += run_result.passed_tests;
                        container.failed_tests += run_result.failed_tests;
                        if run_result.failed_tests > 0 {
                            container.fail_suite(&path);
                        }
                    }
                    Err(error) => {
                        // We failed evaluation... fail the test suite and output the reason
                        println!("{} {}", " FAIL ".white().on_red().bold(), path);
                        let stack_trace = get_stack_trace(&error, Some(path.to_string()));
                        println!(
                            "{}",
                            get_stack_trace_output(
                                "\t\tUnexpected error ocurred when running tests.".to_string(),
                                &stack_trace,
                            )
                            .red()
                        );
                        let mut container = test_container.lock().unwrap();
                        container.fail_suite(&path);
                    }
                }
            }
            Err(error) => {
                // We failed the AST step... fail the test suite and output the reason
                // Note that to get a proper stack trace, we need to convert the ParseError into a EvalAltResult
                let ParseError(error_type, position) = error;
                let rhai_error = rhai::EvalAltResult::ErrorParsing(*error_type, position);
                println!("{} {}", " FAIL ".white().on_red().bold(), path);
                let stack_trace = get_stack_trace(&Box::new(rhai_error), Some(path.to_string()));
                println!(
                    "{}",
                    get_stack_trace_output(
                        "\t\tUnexpected error ocurred when compiling tests.".to_string(),
                        &stack_trace,
                    )
                    .red()
                );
                let mut container = test_container.lock().unwrap();
                container.fail_suite(&path);
            }
        }
    }
    let end_time = Instant::now();

    // Print the coverage if we've opted into it
    if config_shared.lock().unwrap().coverage.unwrap_or_default() {
        test_coverage_container.lock().unwrap().print_results();
    }

    // Print the results
    test_container.lock().unwrap().print_results();

    let elapsed_time = end_time - start_time;

    // Include the amount of time to run the tests
    let time_string = if elapsed_time.as_secs_f64() < 1.0 {
        format!("{:.2} ms", elapsed_time.as_secs_f64() * 1000.0)
    } else {
        format!("{:.2} s", elapsed_time.as_secs_f64())
    };

    println!("Time:        {}", time_string);

    if !is_watch_mode && test_container.lock().unwrap().has_failed_suites() {
        exit(1);
    }
}

/// Clear the terminal screen completely
fn clear_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", "cls"]).status().unwrap();
    } else {
        Command::new("clear").status().unwrap();
    }
}
