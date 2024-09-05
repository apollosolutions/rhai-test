use colored::*;
use std::collections::HashMap;
use tabled::{settings::Style, Table, Tabled};

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

    #[tabled(rename = "Uncovered Line #s")]
    uncovered_lines: String,
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

    pub fn add_branch(&mut self, source: String, line_number: i64) {
        self.maybe_add_source(&source);
        let key = TestCoverageContainer::get_statement_key(&source, &line_number);

        self.sources
            .get_mut(&source)
            .unwrap()
            .branches
            .entry(key)
            .or_insert(BranchCoverage {
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

    pub fn branch_called(&mut self, source: String, line_number: i64) {
        let key = TestCoverageContainer::get_statement_key(&source, &line_number);

        self.sources
            .get_mut(&source)
            .unwrap()
            .branches
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
            let percent_branches = {
                let total_branches = coverage_source.branches.len();
                let hit_branches = coverage_source
                    .branches
                    .iter()
                    .filter(|(_, branch)| branch.is_hit)
                    .count();
                let percent = (hit_branches as f64 / total_branches as f64) * 100.0;
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
                branches: percent_branches.to_string(),
                functions: percent_functions.to_string(),
                uncovered_lines: uncovered_lines.to_string(),
            });
        });

        let table = Table::new(report_data).with(Style::modern()).to_string();
        println!("\n\n{}", table);
    }
}
