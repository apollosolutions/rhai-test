mod expector;
mod extensions;
mod test_container;

use clap::Parser;
use expector::Expector;
use glob::glob;
use rhai::{Engine, FnPtr};
use serde::Deserialize;
use std::fs::{self};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use test_container::TestContainer;

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
    let mut engine = Engine::new();

    engine
        .register_type_with_name::<Expector>("Expector")
        .register_fn("expect", Expector::new)
        .register_fn("not", Expector::not)
        .register_fn("to_be", Expector::to_be)
        .register_fn("to_match", Expector::to_match);

    extensions::apollo::register_rhai_functions_and_types(&mut engine);
    extensions::helpers::register_rhai_functions_and_types(&mut engine);

    extensions::apollo::register_mocking_functions(&mut engine);

    for path in &test_files {
        println!("{}", path);
        let test_file_content = fs::read_to_string(path).expect("Unable to read rhai test file");

        let cloned_container = test_container.clone();
        let cloned_path = path.clone();

        let test = move |test_name: &str, func: FnPtr| {
            cloned_container
                //.borrow_mut()
                .lock()
                .unwrap()
                .add_test(test_name, func, &cloned_path);
        };

        engine.register_fn("test", test);

        match engine.compile(&test_file_content) {
            Ok(ast) => match engine.eval::<()>(&test_file_content) {
                Ok(()) => {
                    test_container
                        .lock()
                        .unwrap()
                        .run_tests(&engine, &ast, &path);
                }
                Err(err) => {
                    println!("Eval error: {}", err);
                }
            },
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
