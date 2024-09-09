test("Should not throw an error when clients header are provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";
    request.headers["apollographql-client-version"] = "1.0";

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    expect(execute).not().to_throw();
});