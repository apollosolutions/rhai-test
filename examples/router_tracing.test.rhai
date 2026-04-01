// Tests for router_tracing.rhai
// Uses get_router_service_request() and get_router_service_response() —
// neither is exercised by any existing example test.

test("Should generate a request ID when none is provided", ||{
    let request = apollo_mocks::get_router_service_request();

    import "router_tracing" as tracing;
    tracing::router_request(request);

    // After calling the function, x-request-id should exist
    expect(request.headers["x-request-id"]).to_exist();
});

test("Should generate a request ID that matches UUID format", ||{
    let request = apollo_mocks::get_router_service_request();

    import "router_tracing" as tracing;
    tracing::router_request(request);

    expect(request.headers["x-request-id"]).to_match("generated-.{8}-.{4}-.{4}-.{4}-.{12}");
});

test("Should log info when generating a new request ID", ||{
    let request = apollo_mocks::get_router_service_request();

    import "router_tracing" as tracing;
    tracing::router_request(request);

    expect(log_info).to_log_message("Generated new request ID");
});

test("Should preserve a client-provided request ID", ||{
    let request = apollo_mocks::get_router_service_request();
    request.headers["x-request-id"] = "client-abc-123";

    import "router_tracing" as tracing;
    tracing::router_request(request);

    // Should not have overwritten the client value
    expect(request.headers["x-request-id"]).to_be("client-abc-123");
});

test("Should log info when using client-provided request ID", ||{
    let request = apollo_mocks::get_router_service_request();
    request.headers["x-request-id"] = "client-abc-123";

    import "router_tracing" as tracing;
    tracing::router_request(request);

    expect(log_info).to_log_message("Using client-provided request ID");
});

test("Should add fallback x-request-id on router response when header is absent", ||{
    let response = apollo_mocks::get_router_service_response();

    import "router_tracing" as tracing;
    tracing::router_response(response);

    expect(response.headers["x-request-id"]).to_be("unknown");
});

test("Should log a warning when no request ID found on response", ||{
    let response = apollo_mocks::get_router_service_response();

    import "router_tracing" as tracing;
    tracing::router_response(response);

    expect(log_warn).to_log_message("No request ID found on response context");
});
