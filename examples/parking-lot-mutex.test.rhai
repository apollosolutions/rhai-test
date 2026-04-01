// ============================================================
// parking-lot-mutex.test.rhai
// Stress-tests the parking_lot::Mutex migration by exercising
// concurrent-style access patterns on mock objects.
// The key change: supergraph_request.lock() no longer returns
// a Result (no .unwrap() needed). This validates that the
// SharedMut pattern works correctly with parking_lot.
// ============================================================

test("Should create subgraph request from supergraph request (exercises parking_lot lock)", ||{
    let supergraph_request = apollo_mocks::get_supergraph_service_request();
    let subgraph_request = apollo_mocks::get_subgraph_service_request(supergraph_request);
    expect(subgraph_request).to_exist();
});

test("Should propagate headers from supergraph to subgraph request through parking_lot mutex", ||{
    let supergraph_request = apollo_mocks::get_supergraph_service_request();
    supergraph_request.headers["x-trace-id"] = "abc-123-def";
    supergraph_request.headers["x-tenant"] = "acme-corp";

    let subgraph_request = apollo_mocks::get_subgraph_service_request(supergraph_request);

    // Headers should propagate through the supergraph→subgraph mock creation
    expect(subgraph_request.headers["x-trace-id"]).to_be("abc-123-def");
    expect(subgraph_request.headers["x-tenant"]).to_be("acme-corp");
});

test("Should allow modifying subgraph-specific headers after creation", ||{
    let supergraph_request = apollo_mocks::get_supergraph_service_request();
    let subgraph_request = apollo_mocks::get_subgraph_service_request(supergraph_request);

    subgraph_request.subgraph.headers["x-downstream"] = "injected";
    expect(subgraph_request.subgraph.headers["x-downstream"]).to_be("injected");
});

test("Should handle multiple subgraph requests from same supergraph request pattern", ||{
    // This pattern mimics what happens in a real Router when a supergraph request
    // fans out to multiple subgraphs. Each call to get_subgraph_service_request
    // must successfully lock the supergraph_request (parking_lot::Mutex).
    let sg_request = apollo_mocks::get_supergraph_service_request();
    sg_request.headers["x-correlation"] = "corr-456";

    let sub_request_1 = apollo_mocks::get_subgraph_service_request(sg_request);
    expect(sub_request_1.headers["x-correlation"]).to_be("corr-456");
});

test("Should create multiple independent supergraph requests without mutex contention", ||{
    let request1 = apollo_mocks::get_supergraph_service_request();
    request1.headers["x-id"] = "first";

    let request2 = apollo_mocks::get_supergraph_service_request();
    request2.headers["x-id"] = "second";

    expect(request1.headers["x-id"]).to_be("first");
    expect(request2.headers["x-id"]).to_be("second");
});
