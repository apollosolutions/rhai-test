test("testing", ||{
    import "client_id" as client_id;
    let request = apollo_mocks::get_supergraph_service_request();

    const execute = || {client_id::process_request(request);};

    expect(execute).to_throw();
});