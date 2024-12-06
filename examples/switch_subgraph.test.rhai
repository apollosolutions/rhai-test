test("Should switch subgraph url when in list of operations", ||{
    let supergraph_request = apollo_mocks::get_supergraph_service_request();
    supergraph_request.body.operation_name = "operation1";
    let request = apollo_mocks::get_subgraph_service_request(supergraph_request);

    import "switch_subgraph" as switch_subgraph;
    switch_subgraph::switch_subgraph_url(request);

    expect(request.subgraph.uri).to_be("/my-api");
});

test("Should not switch subgraph url when not in list of operations", ||{
    let supergraph_request = apollo_mocks::get_supergraph_service_request();
    supergraph_request.body.operation_name = "operation999";
    let request = apollo_mocks::get_subgraph_service_request(supergraph_request);

    import "switch_subgraph" as switch_subgraph;
    switch_subgraph::switch_subgraph_url(request);

    expect(request.subgraph.uri).not().to_be("/my-api");
});