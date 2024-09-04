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

pub struct FileCoverageModuleResolver {
    base_path: PathBuf,
    test_coverage_container: Arc<Mutex<TestCoverageContainer>>,
}

impl FileCoverageModuleResolver {
    pub fn new(
        base_path: impl Into<PathBuf>,
        test_coverage_container: Arc<Mutex<TestCoverageContainer>>,
    ) -> Self {
        Self {
            base_path: base_path.into(),
            test_coverage_container,
        }
    }

    pub fn get_file_path(&self, path: &str, source_path: Option<&Path>) -> PathBuf {
        let path = Path::new(path);

        let mut file_path: PathBuf;

        if path.is_relative() {
            file_path = self.base_path.clone();
            file_path.push(path);
        } else {
            file_path = path.into();
        }

        file_path.set_extension("rhai"); // Force extension
        file_path
    }
}

impl ModuleResolver for FileCoverageModuleResolver {
    // TODO: Need to re-implement the file caching cause this is probably a bottleneck
    // TODO: Using this module resolver is like 2x slower compared to normal resolver... but it seems to be the instrumenting, not the module resolve
    // TODO: But this could also be that due to a lack of caching, it is re-instrumenting the module for every test
    fn resolve(
        &self,
        engine: &Engine,
        source: Option<&str>,
        path: &str,
        pos: Position,
    ) -> Result<rhai::Shared<rhai::Module>, Box<rhai::EvalAltResult>> {
        let global = &mut engine.new_global_runtime_state();
        let scope = &mut Scope::new();
        let source_path = global
            .source()
            .or(source)
            .and_then(|p| Path::new(p).parent());
        let file_path = self.get_file_path(path, source_path);

        let mut contents = fs::read_to_string(file_path)
            .map_err(|_| Box::new(EvalAltResult::ErrorModuleNotFound(path.to_string(), pos)))?;

        contents = contents
            .lines()
            .enumerate()
            .map(|(i, line)| instrument_line(i, line, path, self.test_coverage_container.clone()))
            .collect::<Vec<_>>()
            .join("\n");

        //println!("{}", contents);

        let mut ast = engine.compile(&contents).map_err(|err| {
            Box::new(EvalAltResult::ErrorInModule(
                path.to_string(),
                err.into(),
                pos,
            ))
        })?;
        ast.set_source(path);

        let m = Module::eval_ast_as_new_raw(engine, scope, global, &ast)
            .map_err(|err| Box::new(EvalAltResult::ErrorInModule(path.to_string(), err, pos)))?
            .into();

        Ok(m)
    }
}

fn instrument_line(
    i: usize,
    line: &str,
    path: &str,
    test_coverage_container: Arc<Mutex<TestCoverageContainer>>,
) -> String {
    let mut result = String::from(line);

    if let Some(captures) = Regex::new(r#"fn (.+?)\(.*?\)\s*?\{"#)
        .unwrap()
        .captures(line)
    {
        // Instrument Functions
        if let Some(matched) = captures.get(1) {
            let function_name = matched.as_str();

            test_coverage_container.lock().unwrap().add_function(
                function_name.to_string(),
                path.to_string(),
                (i as i64) + 1,
            );

            let instrumentation = format!(
                "rhai_test_coverage_instrument_function(\"{}\",\"{}\",{} );",
                function_name,
                path,
                (i as i64) + 1
            );
            result = format!("{} {}", line, instrumentation);
        }
    } else if (Regex::new(r#".+?\(.*?\);"#).unwrap().is_match(line)) {
        // Function Call Statements
        test_coverage_container
            .lock()
            .unwrap()
            .add_statement(path.to_string(), (i as i64) + 1);

        let instrumentation = format!(
            "rhai_test_coverage_instrument_statement(\"{}\",{} );",
            path,
            (i as i64) + 1
        );
        result = format!("{} {}", line, instrumentation);
    } else if (Regex::new(r#"(let )?.+?=.+?;"#).unwrap().is_match(line)) {
        // Variable declarations
        test_coverage_container
            .lock()
            .unwrap()
            .add_statement(path.to_string(), (i as i64) + 1);

        let instrumentation = format!(
            "rhai_test_coverage_instrument_statement(\"{}\",{} );",
            path,
            (i as i64) + 1
        );
        result = format!("{} {}", line, instrumentation);
    } else if (Regex::new(r#"(else|else if|if).+?\{"#)
        .unwrap()
        .is_match(line))
    {
        // Branches
        let instrumentation = format!(
            "rhai_test_coverage_instrument_branch(\"{}\",{} );",
            path,
            (i as i64) + 1
        );
        result = format!("{} {}", line, instrumentation);
    } else if (Regex::new(r#"throw.+?\{"#).unwrap().is_match(line)) {
        // throws
        test_coverage_container
            .lock()
            .unwrap()
            .add_statement(path.to_string(), (i as i64) + 1);

        let instrumentation = format!(
            "rhai_test_coverage_instrument_statement(\"{}\",{} );",
            path,
            (i as i64) + 1
        );
        result = format!("{} {}", instrumentation, line);
    }
    result
}

#[derive(Debug)]
struct TestCoverageSource {
    pub name: String,
    statements: HashMap<String, StatementCoverage>,
    branches: HashMap<String, BranchCoverage>,
    functions: HashMap<String, FunctionCoverage>,
}

#[derive(Debug)]
struct FunctionCoverage {
    pub function_name: String,
    pub source: String,
    pub line_number: i64,
    pub is_hit: bool,
}

#[derive(Debug)]
struct StatementCoverage {
    pub source: String,
    pub line_number: i64,
    pub is_hit: bool,
}

#[derive(Debug)]
struct BranchCoverage {
    pub source: String,
    pub line_number: i64,
    pub is_hit: bool,
}

#[derive(Debug)]
pub struct TestCoverageContainer {
    sources: HashMap<String, TestCoverageSource>,
}

impl TestCoverageContainer {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    fn maybe_add_source(&mut self, source: &String) {
        let name = String::from(source);

        if !self.sources.contains_key(&name.clone()) {
            self.sources.insert(
                name.clone(),
                TestCoverageSource {
                    name,
                    functions: HashMap::new(),
                    statements: HashMap::new(),
                    branches: HashMap::new(),
                },
            );
        }
    }

    pub fn add_function(&mut self, function_name: String, source: String, line_number: i64) {
        self.maybe_add_source(&source);
        let key = TestCoverageContainer::get_function_key(&function_name, &source, &line_number);

        self.sources
            .get_mut(&source)
            .unwrap()
            .functions
            .entry(key)
            .or_insert(FunctionCoverage {
                function_name,
                source,
                line_number,
                is_hit: false,
            });
    }

    pub fn add_statement(&mut self, source: String, line_number: i64) {
        self.maybe_add_source(&source);
        let key = TestCoverageContainer::get_statement_key(&source, &line_number);

        self.sources
            .get_mut(&source)
            .unwrap()
            .statements
            .entry(key)
            .or_insert(StatementCoverage {
                source,
                line_number,
                is_hit: false,
            });
    }

    pub fn function_called(&mut self, function_name: String, source: String, line_number: i64) {
        let key = TestCoverageContainer::get_function_key(&function_name, &source, &line_number);

        self.sources
            .get_mut(&source)
            .unwrap()
            .functions
            .get_mut(&key)
            .unwrap()
            .is_hit = true;
    }

    pub fn statement_called(&mut self, source: String, line_number: i64) {
        let key = TestCoverageContainer::get_statement_key(&source, &line_number);

        self.sources
            .get_mut(&source)
            .unwrap()
            .statements
            .get_mut(&key)
            .unwrap()
            .is_hit = true;
    }

    fn get_function_key(function_name: &String, source: &String, line_number: &i64) -> String {
        format!("{}-{}-{}", function_name, source, line_number)
    }

    fn get_statement_key(source: &String, line_number: &i64) -> String {
        format!("{}-{}", source, line_number)
    }

    pub fn print_results(&mut self) {
        let mut report_data = Vec::<CoverageReportLine>::new();

        self.sources.iter().for_each(|(_, coverage_source)| {
            let source = &coverage_source.name;
            let percent_functions = {
                let total_functions = coverage_source.functions.len();
                let hit_functions = coverage_source
                    .functions
                    .iter()
                    .filter(|(_, function)| function.is_hit)
                    .count();
                let percent = (hit_functions as f64 / total_functions as f64) * 100.0;
                if percent >= 80.0 {
                    percent.to_string().green()
                } else if percent >= 50.0 {
                    percent.to_string().yellow()
                } else {
                    percent.to_string().red()
                }
            };
            let percent_statements = {
                let total_statements = coverage_source.statements.len();
                let hit_statements = coverage_source
                    .statements
                    .iter()
                    .filter(|(_, statement)| statement.is_hit)
                    .count();
                let percent = (hit_statements as f64 / total_statements as f64) * 100.0;
                if percent >= 80.0 {
                    percent.to_string().green()
                } else if percent >= 50.0 {
                    percent.to_string().yellow()
                } else {
                    percent.to_string().red()
                }
            };
            let uncovered_lines = coverage_source
                .statements
                .iter()
                .filter(|(_, statement)| !statement.is_hit)
                .map(|(_, statement)| statement.line_number.to_string())
                .collect::<Vec<_>>()
                .join(",");

            report_data.push(CoverageReportLine {
                source: source.to_string(),
                statements: percent_statements.to_string(),
                branches: "0".to_string(),
                functions: percent_functions.to_string(),
                lines: "0".to_string(),
                uncovered_lines: uncovered_lines.to_string(),
            });
        });

        let table = Table::new(report_data).with(Style::modern()).to_string();
        println!("\n\n{}", table);
    }
}

#[derive(Tabled)]
struct CoverageReportLine {
    #[tabled(rename = "Source")]
    source: String,

    #[tabled(rename = "% Stmts")]
    statements: String,

    #[tabled(rename = "% Branch")]
    branches: String,

    #[tabled(rename = "% Funcs")]
    functions: String,

    #[tabled(rename = "% Lines")]
    lines: String,

    #[tabled(rename = "Uncovered Line #s")]
    uncovered_lines: String,
}

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
