// Demonstrates the subgraph_request_id correlation pattern Indeed and other
// customers use in production: stash a payload on request.context keyed by
// the subgraph request id, then pull and remove it on the response side.
//
// This file is the "script under test" for subgraph_request_id_correlation.test.rhai;
// the test file imports this module and exercises these hooks against the mocks.
// Use this pair as a copy-paste template for testing your own correlation logic.

fn subgraph_request(request) {
    let rid = request.subgraph_request_id;
    request.context["correlation:" + rid] = #{
        started_at: "2026-04-29T00:00:00Z",
        attempt: 1,
    };
}

fn subgraph_response(response) {
    let rid = response.subgraph_request_id;
    let payload = response.context.remove("correlation:" + rid);
    if payload != () {
        response.headers["x-correlation-started-at"] = payload.started_at;
    }
}
