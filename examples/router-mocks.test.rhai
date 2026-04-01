// ============================================================
// router-mocks.test.rhai
// Tests router service request/response mocks (NOT covered by existing examples)
// Validates that registration::register(engine) properly exposes
// all Router lifecycle types after the macro→module migration.
// ============================================================

test("Should create a router service request mock", ||{
    let request = apollo_mocks::get_router_service_request();
    expect(request).to_exist();
});

test("Should be able to read headers on router service request", ||{
    let request = apollo_mocks::get_router_service_request();
    // Headers should exist as an object even if empty
    expect(request.headers).to_exist();
});

test("Should be able to set and read headers on router service request", ||{
    let request = apollo_mocks::get_router_service_request();
    request.headers["x-custom-header"] = "test-value";
    expect(request.headers["x-custom-header"]).to_be("test-value");
});

test("Should create a router service response mock", ||{
    let response = apollo_mocks::get_router_service_response();
    expect(response).to_exist();
});

test("Should be able to set and read headers on router service response", ||{
    let response = apollo_mocks::get_router_service_response();
    response.headers["x-response-header"] = "response-value";
    expect(response.headers["x-response-header"]).to_be("response-value");
});
