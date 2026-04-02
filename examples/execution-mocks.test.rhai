// ============================================================
// execution-mocks.test.rhai
// Tests execution service request/response mocks (NOT covered by existing examples)
// Validates parking_lot::Mutex compatibility with execution lifecycle types.
// ============================================================

test("Should create an execution service request mock", ||{
    let request = apollo_mocks::get_execution_service_request();
    expect(request).to_exist();
});

test("Should be able to read headers on execution service request", ||{
    let request = apollo_mocks::get_execution_service_request();
    expect(request.headers).to_exist();
});

test("Should be able to set headers on execution service request", ||{
    let request = apollo_mocks::get_execution_service_request();
    request.headers["x-execution-header"] = "exec-value";
    expect(request.headers["x-execution-header"]).to_be("exec-value");
});

test("Should create an execution service response mock", ||{
    let response = apollo_mocks::get_execution_service_response();
    expect(response).to_exist();
});

test("Should be able to set headers on execution service response", ||{
    let response = apollo_mocks::get_execution_service_response();
    response.headers["x-exec-response"] = "done";
    expect(response.headers["x-exec-response"]).to_be("done");
});
