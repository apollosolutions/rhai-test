// ============================================================
// supergraph-response.test.rhai
// Tests supergraph response mock (NOT covered by existing examples)
// Also exercises subgraph response mock.
// ============================================================

test("Should create a supergraph service response mock", ||{
    let response = apollo_mocks::get_supergraph_service_response();
    expect(response).to_exist();
});

test("Should be able to read headers on supergraph service response", ||{
    let response = apollo_mocks::get_supergraph_service_response();
    expect(response.headers).to_exist();
});

test("Should be able to set headers on supergraph service response", ||{
    let response = apollo_mocks::get_supergraph_service_response();
    response.headers["x-sg-response"] = "sg-value";
    expect(response.headers["x-sg-response"]).to_be("sg-value");
});

test("Should create a subgraph service response mock", ||{
    let response = apollo_mocks::get_subgraph_service_response();
    expect(response).to_exist();
});

test("Should be able to set headers on subgraph service response", ||{
    let response = apollo_mocks::get_subgraph_service_response();
    response.headers["x-sub-response"] = "sub-value";
    expect(response.headers["x-sub-response"]).to_be("sub-value");
});
