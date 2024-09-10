use colored::Colorize;
use rhai::{EvalAltResult, Map, Position};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct StackTraceDetail {
    pub message: String,
    pub status_code: String,
    pub position: Position,
    pub source: String,
}

impl StackTraceDetail {
    pub fn new(message: String, status_code: String, position: Position, source: String) -> Self {
        Self {
            message,
            status_code,
            position,
            source,
        }
    }
}

pub fn get_stack_trace(
    error: &Box<EvalAltResult>,
    parent_source: Option<String>,
) -> Vec<StackTraceDetail> {
    let mut stack_trace = Vec::<StackTraceDetail>::new();

    // TODO: Add rest of arms for error types
    match **error {
        rhai::EvalAltResult::ErrorSystem(ref message, ..) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Unknown System Error: {}", message.clone()),
                "".to_string(),
                Position::NONE,
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorVariableExists(ref name, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!(
                    "Shadowing of an existing variable disallowed: {}",
                    name.clone()
                ),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorForbiddenVariable(ref name, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Forbidden variable name: {}", name.clone()),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorVariableNotFound(ref name, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Access of an unknown variable: {}", name.clone()),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorPropertyNotFound(ref name, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Access of an unknown object map property: {}", name.clone()),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorIndexNotFound(ref name, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Access of an invalid index: {}", name.clone()),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorInFunctionCall(ref name, ref source, ref inner, ref position) => {
            let file = if !source.is_empty() {
                format!("{}.rhai", source)
            } else {
                String::new()
            };
            stack_trace.push(StackTraceDetail::new(
                format!("Error in function call: {}", name),
                "".to_string(),
                position.clone(),
                file.clone(),
            ));

            stack_trace.extend(get_stack_trace(&inner, Some(file.clone())));
        }

        rhai::EvalAltResult::ErrorInModule(ref name, ref inner, ref position) => {
            let file = format!("{}.rhai", name);
            stack_trace.push(StackTraceDetail::new(
                format!("Error in module: {}", name),
                "".to_string(),
                position.clone(),
                "".to_string(),
            ));

            stack_trace.extend(get_stack_trace(&inner, Some(file.clone())));
        }

        rhai::EvalAltResult::ErrorModuleNotFound(ref module_name, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Module not found: {}. Hint: If you're importing a module in a test file, don't forget to use inline imports scoped to the function you're using the import in.", module_name.clone().to_string()),
                "".to_string(),
                position.clone(),
                "".to_string(),
            ));
        }
        rhai::EvalAltResult::ErrorRuntime(ref error_token, ref position) => {
            if let Some(map) = error_token.read_lock::<Map>() {
                let message = map.get("message").unwrap().to_string();
                let status = map.get("status").unwrap().to_string();

                stack_trace.push(StackTraceDetail::new(
                    message,
                    status.clone().to_string(),
                    position.clone(),
                    parent_source.unwrap_or_default(),
                ));
            } else {
                stack_trace.push(StackTraceDetail::new(
                    error_token.clone().to_string(),
                    "".to_string(),
                    position.clone(),
                    parent_source.unwrap_or_default(),
                ));
            }
        }
        rhai::EvalAltResult::ErrorFunctionNotFound(ref function_signature, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Function not found: {}.", function_signature),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorUnboundThis(ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Access to `this` that is not bound."),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorMismatchDataType(ref requested, ref actual, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!(
                    "Data is not of the required type. Requested: {} actual: {}",
                    requested, actual
                ),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorMismatchOutputType(ref requested, ref actual, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!(
                    "Returned type is not the same as the required output type. Requested: {} actual: {}",
                    requested, actual
                ),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorIndexingType(ref name, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!(
                    "Tried to index into a type that has no indexer function defined: {}",
                    name
                ),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorArrayBounds(ref elements, ref index, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Array access out-of-bounds: {} {}", elements, index),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorStringBounds(ref elements, ref index, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("String indexing out-of-bounds: {} {}", elements, index),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorBitFieldBounds(ref elements, ref index, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Bit-field indexing out-of-bounds: {} {}", elements, index),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorFor(ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("`for` statement encountered a type that is not iterable"),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorDataRace(ref name, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Data race detected when accessing variable: {}", name),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorNonPureMethodCallOnConstant(ref name, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Called a non-pure method on constant: {}", name),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorAssignmentToConstant(ref name, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Assignment to constant variable: {}", name),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorDotExpr(ref name, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Inappropriate property access: {}", name),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorArithmetic(ref message, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Arithmetic error encountered: {}", message),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorTooManyOperations(ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Number of operations over maximum limit"),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorTooManyVariables(ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Number of variables over maximum limit"),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorTooManyModules(ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Number of modules over maximum limit"),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorStackOverflow(ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Call stack over maximum limit"),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::ErrorTerminated(ref token, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("The script is prematurely terminated with: {}", token),
                "".to_string(),
                position.clone(),
                parent_source.unwrap_or_default(),
            ));
        }
        rhai::EvalAltResult::LoopBreak(..) => {
            // Not actually an error
        }
        rhai::EvalAltResult::Return(..) => {
            // Not actually an error
        }
        rhai::EvalAltResult::Exit(..) => {
            // Not actually an error
        }
        rhai::EvalAltResult::ErrorParsing(ref syntax_error, position) => {
            match syntax_error {
                rhai::ParseErrorType::UnexpectedEOF => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Unexpected end of file",),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::BadInput(ref lex_error) => match lex_error {
                    rhai::LexError::UnexpectedInput(symbol) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!("Parsing Error: Unexpected symbol: {}", symbol),
                            "".to_string(),
                            position.clone(),
                            parent_source.unwrap_or_default(),
                        ));
                    }
                    rhai::LexError::UnterminatedString => {
                        stack_trace.push(StackTraceDetail::new(
                            format!("Parsing Error: String literal not terminated before new-line or EOF."),
                            "".to_string(),
                            position.clone(),
                            parent_source.unwrap_or_default(),
                        ));
                    }
                    rhai::LexError::StringTooLong(..) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!("Parsing Error: identifier or string literal longer than the maximum allowed length."),
                            "".to_string(),
                            position.clone(),
                            parent_source.unwrap_or_default(),
                        ));
                    }
                    rhai::LexError::MalformedEscapeSequence(sequence) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!("Parsing Error: string/character/numeric escape sequence is in an invalid format: {}", sequence),
                            "".to_string(),
                            position.clone(),
                            parent_source.unwrap_or_default(),
                        ));
                    }
                    rhai::LexError::MalformedNumber(number) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!(
                                "Parsing Error: numeric literal is in an invalid format: {}",
                                number
                            ),
                            "".to_string(),
                            position.clone(),
                            parent_source.unwrap_or_default(),
                        ));
                    }
                    rhai::LexError::MalformedChar(char) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!(
                                "Parsing Error: character literal is in an invalid format: {}",
                                char
                            ),
                            "".to_string(),
                            position.clone(),
                            parent_source.unwrap_or_default(),
                        ));
                    }
                    rhai::LexError::MalformedIdentifier(identifier) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!(
                                "Parsing Error: identifier is in an invalid format: {}",
                                identifier
                            ),
                            "".to_string(),
                            position.clone(),
                            parent_source.unwrap_or_default(),
                        ));
                    }
                    rhai::LexError::ImproperSymbol(a, b) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!("Parsing Error: Bad symbol encountered: {} {}", a, b),
                            "".to_string(),
                            position.clone(),
                            parent_source.unwrap_or_default(),
                        ));
                    }
                    rhai::LexError::Runtime(message) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!("Parsing Error: Runtime error: {}", message),
                            "".to_string(),
                            position.clone(),
                            parent_source.unwrap_or_default(),
                        ));
                    }
                    _ => {
                        println!("\t{}", " Unknown parsing error occurred. ".red());
                    }
                },
                rhai::ParseErrorType::UnknownOperator(ref operator) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: unknown operator encountered: {}", operator),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::MissingToken(ref token, ref description) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Expected token: {} {}", token, description),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::MissingSymbol(ref description) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Expected Symbol: {}", description),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::MalformedIndexExpr(ref description) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Syntax error with expression in indexing brackets `[]`: {}", description),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::MalformedCapture(ref description) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!(
                            "Parsing Error: Syntax error with a capture: {}",
                            description
                        ),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::DuplicatedProperty(ref description) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!(
                            "Parsing Error: Map definition has duplicated property names: {}",
                            description
                        ),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::DuplicatedVariable(ref description) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: variable name duplicated: {}", description),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::WrongSwitchIntegerCase => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: numeric case of `switch` statement is in an appropriate place."),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::WrongSwitchDefaultCase => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: default case of `switch` statement is in an appropriate place."),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::WrongSwitchCaseCondition => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: case condition of `switch` statement is not appropriate"),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::PropertyExpected => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Missing property name for custom type or map"),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::VariableExpected => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Missing variable name after a `let`, `const`, `for` or `catch` keyword."),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::ForbiddenVariable(name) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Forbidden variable name: {}", name),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::Reserved(name) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Reserved symbol: {}", name),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::MismatchedType(requested, actual) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!(
                            "Parsing Error: Type mismatch. Requested: {}, Actual: {}",
                            requested, actual
                        ),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::ExprExpected(expression) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Expression expected: {}", expression),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::WrongDocComment => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: doc-comment defined in an appropriate place"),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::WrongFnDefinition => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: function `fn` defined in an appropriate place"),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::FnDuplicatedDefinition(name, params) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: function defined with a name that conflicts with an existing function: {} {}.", name, params),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::FnMissingName => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Missing a function name after the `fn` keyword."),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::FnMissingParams(name) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!(
                            "Parsing Error: function definition is missing the parameters list: {}",
                            name
                        ),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::FnDuplicatedParam(name, param) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!(
                            "Parsing Error: function definition has duplicated parameters: {} {}",
                            name, param
                        ),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::FnMissingBody(name) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!(
                            "Parsing Error: function definition is missing body: {}",
                            name
                        ),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::WrongExport => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Export statement found not at global level.",),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::AssignmentToConstant(name) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Assignment to a constant variable: {}", name),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::AssignmentToInvalidLHS(message) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Assignment to an inappropriate left-hand-side expression: {}", message),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::VariableExists(name) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Variable is already defined: {}", name),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::VariableUndefined(name) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Variable not found: {}", name),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::ModuleUndefined(name) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Imported module not found: {}", name),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::ExprTooDeep => {
                    stack_trace.push(StackTraceDetail::new(
                        format!(
                            "Parsing Error: Expression exceeding the maximum levels of complexity."
                        ),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::TooManyFunctions => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Number of scripted functions over maximum limit."),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::LiteralTooLarge(data_type, size) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!(
                            "Parsing Error: Literal exceeding the maximum size: {} {}",
                            data_type, size
                        ),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                rhai::ParseErrorType::LoopBreak => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Break statement found not inside a loop.",),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
                ref unknown_error => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Unknown error occurred: {}", unknown_error),
                        "".to_string(),
                        position.clone(),
                        parent_source.unwrap_or_default(),
                    ));
                }
            }
        }
        ref unknown_error => {
            stack_trace.push(StackTraceDetail::new(
                format!("Unknown error occurred: {}", unknown_error),
                "".to_string(),
                unknown_error.position(),
                parent_source.unwrap_or_default(),
            ));
        }
    }

    stack_trace
}

pub fn get_stack_trace_output(message: String, stack_trace: &Vec<StackTraceDetail>) -> String {
    let mut output = String::new();

    output.push_str(&message);
    output.push_str("\n");

    // Iterate over stack trace details in reverse order
    for stack_trace_detail in stack_trace.iter().rev() {
        let source_details = if stack_trace_detail.source != "" {
            format!(
                "({}:{:?}:{:?})",
                stack_trace_detail.source,
                stack_trace_detail.position.line().unwrap_or_default(),
                stack_trace_detail.position.position().unwrap_or_default()
            )
        } else {
            "".to_string()
        };

        writeln!(
            output,
            "\t\t{} {}",
            stack_trace_detail.message, source_details
        )
        .unwrap();
    }

    output
}

pub fn get_inner_most_error(error: &Box<EvalAltResult>) -> &Box<EvalAltResult> {
    let inner_most_error;

    match **error {
        rhai::EvalAltResult::ErrorInFunctionCall(_, _, ref inner, _) => {
            inner_most_error = get_inner_most_error(inner);
        }
        _ => {
            inner_most_error = error;
        }
    }

    inner_most_error
}
