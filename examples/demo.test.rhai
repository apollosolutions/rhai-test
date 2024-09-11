test("test 1", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";

    let execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    try {execute.call();} catch {}

    expect(log_error).to_log_message("No client headers set");
});