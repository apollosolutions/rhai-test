test("Should pass when function throws", ||{
    import "client_id" as client_id;
    let request = apollo_mocks::get_supergraph_service_request();

    const execute = || {client_id::process_request(request);};

    expect(execute).to_throw();
});

test("Should pass when function does not throw", ||{
    import "client_id" as client_id;
    let request = apollo_mocks::get_supergraph_service_request();

    const execute = || {};

    expect(execute).not().to_throw();
});