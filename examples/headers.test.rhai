test("Should be able to modify subgraph requestsvia supergraph request", ||{
    let supergraph_request = apollo_mocks::get_supergraph_service_request();
    supergraph_request.headers["assetid"] = "abc123";
    let subgraph_request = apollo_mocks::get_subgraph_service_request(supergraph_request);

    import "headers" as headers;
    headers::rename_header(subgraph_request);
    

    expect(subgraph_request.subgraph.headers["original_assetid"]).to_be("abc123");
});