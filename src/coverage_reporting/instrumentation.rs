use super::test_coverage_container::TestCoverageContainer;
use regex::Regex;
use std::sync::{Arc, Mutex};

pub fn instrument_line(
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

            result = Regex::new(r#"(?P<c1>fn .+?\(.*?\)\s*?\{)(?P<c2>.*?)"#)
                .unwrap()
                .replace(&result, format!("$c1 {} $c2", instrumentation))
                .to_string();
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
        test_coverage_container
            .lock()
            .unwrap()
            .add_branch(path.to_string(), (i as i64) + 1);

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
