// Tests for subgraph_response_enrichment.rhai
// Uses get_subgraph_service_response() and get_supergraph_service_response() —
// neither is exercised by any existing example test.

test("Should mark subgraph response as processed", ||{
    let response = apollo_mocks::get_subgraph_service_response();

    import "subgraph_response_enrichment" as enrichment;
    enrichment::subgraph_response(response);

    expect(response.headers["x-subgraph-processed"]).to_be("true");
});

test("Should log debug when processing subgraph response", ||{
    let response = apollo_mocks::get_subgraph_service_response();

    import "subgraph_response_enrichment" as enrichment;
    enrichment::subgraph_response(response);

    expect(log_debug).to_log_message("Marked subgraph response as processed");
});

test("Should not throw when processing a subgraph response", ||{
    let response = apollo_mocks::get_subgraph_service_response();

    const execute = || {
        import "subgraph_response_enrichment" as enrichment;
        enrichment::subgraph_response(response);
    };

    expect(execute).not().to_throw();
});

test("Should add x-served-by header on supergraph response", ||{
    let response = apollo_mocks::get_supergraph_service_response();

    import "subgraph_response_enrichment" as enrichment;
    enrichment::supergraph_response(response);

    expect(response.headers["x-served-by"]).to_be("apollo-router");
});

test("Should log info when tagging supergraph response", ||{
    let response = apollo_mocks::get_supergraph_service_response();

    import "subgraph_response_enrichment" as enrichment;
    enrichment::supergraph_response(response);

    expect(log_info).to_log_message("Tagged supergraph response with served-by header");
});

test("Should not throw when processing a supergraph response", ||{
    let response = apollo_mocks::get_supergraph_service_response();

    const execute = || {
        import "subgraph_response_enrichment" as enrichment;
        enrichment::supergraph_response(response);
    };

    expect(execute).not().to_throw();
});
