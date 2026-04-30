// Exercises the subgraph_request_id correlation pattern:
//   - stash a payload on request.context keyed on subgraph_request_id
//   - mirror the id onto the mock subgraph response
//   - pull + remove the payload from response.context on the way out
//
// Covers: writable response.subgraph_request_id, context.remove() on both
// request.context and response.context.

test("Should allow writing to subgraph_request_id on subgraph response mock", ||{
    let mock_resp = apollo_mocks::get_subgraph_service_response();
    let original = mock_resp.subgraph_request_id;
    mock_resp.subgraph_request_id = "custom-id-xyz";

    expect(mock_resp.subgraph_request_id).to_be("custom-id-xyz");
    expect(mock_resp.subgraph_request_id).not().to_be(original);
});

test("Should remove an entry from response context", ||{
    let mock_resp = apollo_mocks::get_subgraph_service_response();
    mock_resp.context["correlation:trace"] = #{ trace_id: "abc" };

    let removed = mock_resp.context.remove("correlation:trace");

    expect(removed.trace_id).to_be("abc");
    expect(mock_resp.context["correlation:trace"]).to_be(());
});

test("Should remove an entry from request context", ||{
    let supergraph_request = apollo_mocks::get_supergraph_service_request();
    let mock_req = apollo_mocks::get_subgraph_service_request(supergraph_request);
    mock_req.context["correlation:trace"] = #{ trace_id: "def" };

    let removed = mock_req.context.remove("correlation:trace");

    expect(removed.trace_id).to_be("def");
    expect(mock_req.context["correlation:trace"]).to_be(());
});

test("Should return unit when removing a missing context key", ||{
    let mock_resp = apollo_mocks::get_subgraph_service_response();
    let removed = mock_resp.context.remove("never-set");

    expect(removed).to_be(());
});

test("Should support the full subgraph_request_id correlation pattern", ||{
    // 1. Incoming subgraph request: stash a correlation payload keyed on the
    //    subgraph_request_id.
    let supergraph_request = apollo_mocks::get_supergraph_service_request();
    let mock_req = apollo_mocks::get_subgraph_service_request(supergraph_request);
    let rid = mock_req.subgraph_request_id;
    mock_req.context["correlation:" + rid] = #{ trace_id: "abc-123", span: "s1" };

    // 2. Outgoing subgraph response: writes back with the same id, pulls the
    //    payload, removes it. In a real script the payload would be on
    //    request.context; for this mock example we copy it onto
    //    response.context to simulate the merge the Router does between
    //    request and response contexts for subgraph calls.
    let mock_resp = apollo_mocks::get_subgraph_service_response();
    mock_resp.subgraph_request_id = rid;
    mock_resp.context["correlation:" + rid] = mock_req.context["correlation:" + rid];

    let payload = mock_resp.context.remove("correlation:" + rid);

    expect(mock_resp.subgraph_request_id).to_be(rid);
    expect(payload.trace_id).to_be("abc-123");
    expect(payload.span).to_be("s1");
    expect(mock_resp.context["correlation:" + rid]).to_be(());
});

// The following two tests demonstrate the idiomatic rhai-test pattern: import
// a sibling .rhai source file (the script under test) and exercise its hooks
// against the mocks. Use this shape as a copy-paste template for testing your
// own correlation logic.

test("Should stash correlation payload on request context via subgraph_request hook", ||{
    let supergraph_request = apollo_mocks::get_supergraph_service_request();
    let request = apollo_mocks::get_subgraph_service_request(supergraph_request);
    let rid = request.subgraph_request_id;

    import "subgraph_request_id_correlation" as correlation;
    correlation::subgraph_request(request);

    let stashed = request.context["correlation:" + rid];
    expect(stashed.attempt).to_be(1);
    expect(stashed.started_at).to_be("2026-04-29T00:00:00Z");
});

test("Should pull and remove correlation payload via subgraph_response hook", ||{
    let response = apollo_mocks::get_subgraph_service_response();
    response.subgraph_request_id = "test-rid-9";
    response.context["correlation:test-rid-9"] = #{
        started_at: "2026-04-29T00:00:00Z",
        attempt: 1,
    };

    import "subgraph_request_id_correlation" as correlation;
    correlation::subgraph_response(response);

    expect(response.headers["x-correlation-started-at"]).to_be("2026-04-29T00:00:00Z");
    expect(response.context["correlation:test-rid-9"]).to_be(());
});
