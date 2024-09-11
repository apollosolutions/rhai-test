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

test("test 2", ||{
    let router_request = apollo_mocks::get_router_service_request();
    let router_response = apollo_mocks::get_router_service_response();
    let supergraph_request = apollo_mocks::get_supergraph_service_request();
    let supergraph_response = apollo_mocks::get_supergraph_service_response();
    let execution_request = apollo_mocks::get_execution_service_request();
    let execution_response = apollo_mocks::get_execution_service_response();
    let subgraph_request = apollo_mocks::get_subgraph_service_request();
    let subgraph_response = apollo_mocks::get_subgraph_service_response();

    expect("a").to_be("a");
});
