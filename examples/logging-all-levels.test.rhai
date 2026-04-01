// ============================================================
// logging-all-levels.test.rhai
// Tests all 5 logging levels (trace, debug, info, warn, error)
// to confirm they're properly registered after the engine
// refactor from macros to registration::register().
// ============================================================

test("Should capture log_trace calls", ||{
    log_trace("trace message for testing");
    expect(log_trace).to_log();
});

test("Should capture log_trace with specific message", ||{
    log_trace("specific trace msg");
    expect(log_trace).to_log_message("specific trace msg");
});

test("Should capture log_debug calls", ||{
    log_debug("debug message for testing");
    expect(log_debug).to_log();
});

test("Should capture log_debug with specific message", ||{
    log_debug("specific debug msg");
    expect(log_debug).to_log_message("specific debug msg");
});

test("Should capture log_info calls", ||{
    log_info("info message for testing");
    expect(log_info).to_log();
});

test("Should capture log_info with specific message", ||{
    log_info("specific info msg");
    expect(log_info).to_log_message("specific info msg");
});

test("Should capture log_warn calls", ||{
    log_warn("warn message for testing");
    expect(log_warn).to_log();
});

test("Should capture log_warn with specific message", ||{
    log_warn("specific warn msg");
    expect(log_warn).to_log_message("specific warn msg");
});

test("Should capture log_error calls", ||{
    log_error("error message for testing");
    expect(log_error).to_log();
});

test("Should capture log_error with specific message", ||{
    log_error("specific error msg");
    expect(log_error).to_log_message("specific error msg");
});
