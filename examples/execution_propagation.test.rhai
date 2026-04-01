// Tests for execution_propagation.rhai
// Uses get_execution_service_request() — not exercised by any existing example.

test("Should log info when a valid tenant ID is present", ||{
    let request = apollo_mocks::get_execution_service_request();
    request.headers["x-tenant-id"] = "acme-corp";

    import "execution_propagation" as exec;
    exec::execution_request(request);

    expect(log_info).to_log_message("Propagating tenant: acme-corp");
});

test("Should not throw when a valid tenant header is provided", ||{
    let request = apollo_mocks::get_execution_service_request();
    request.headers["x-tenant-id"] = "acme-corp";

    const execute = || {
        import "execution_propagation" as exec;
        exec::execution_request(request);
    };

    expect(execute).not().to_throw();
});

test("Should throw with status 400 when tenant ID header is blank", ||{
    let request = apollo_mocks::get_execution_service_request();
    request.headers["x-tenant-id"] = "";

    const execute = || {
        import "execution_propagation" as exec;
        exec::execution_request(request);
    };

    expect(execute).to_throw_status(400);
});

test("Should throw with correct message when tenant ID header is blank", ||{
    let request = apollo_mocks::get_execution_service_request();
    request.headers["x-tenant-id"] = "";

    const execute = || {
        import "execution_propagation" as exec;
        exec::execution_request(request);
    };

    expect(execute).to_throw_message("x-tenant-id header must not be blank");
});

test("Should log error when tenant ID header is blank", ||{
    let request = apollo_mocks::get_execution_service_request();
    request.headers["x-tenant-id"] = "";

    try {
        import "execution_propagation" as exec;
        exec::execution_request(request);
    } catch {}

    expect(log_error).to_log_message("Tenant ID header is blank");
});

test("Should log warn when no tenant ID header present", ||{
    let request = apollo_mocks::get_execution_service_request();

    import "execution_propagation" as exec;
    exec::execution_request(request);

    expect(log_warn).to_log_message("No x-tenant-id header present");
});

test("Should not throw when no tenant ID header is present", ||{
    let request = apollo_mocks::get_execution_service_request();

    const execute = || {
        import "execution_propagation" as exec;
        exec::execution_request(request);
    };

    expect(execute).not().to_throw();
});
