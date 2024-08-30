/*test("Should throw an error when no client headers are provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);};

    expect(execute).to_throw();
});*/

test("Should throw an error with message when no client headers are provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);};

    expect(execute).to_throw_message("No client headers set. Please provide headers: apollographql-client-name and apollographql-client-version");
});
/* 
test("Should throw an error when apollographql-client-version header is not provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    expect(execute).to_throw();
});

test("Should throw an error when apollographql-client-name header is not provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-version"] = "1.0";

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    expect(execute).to_throw();
});

test("Should not throw an error when clients header are provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";
    request.headers["apollographql-client-version"] = "1.0";

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    expect(execute).not().to_throw();
});*/