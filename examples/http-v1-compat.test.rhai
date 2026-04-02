// ============================================================
// http-v1-compat.test.rhai
// Tests that the http 0.2 → 1.0 bump doesn't regress
// header/method/URI handling in mock objects.
// ============================================================

test("Should handle standard HTTP headers on supergraph request", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["content-type"] = "application/json";
    request.headers["authorization"] = "Bearer token123";
    request.headers["accept"] = "application/graphql-response+json";

    expect(request.headers["content-type"]).to_be("application/json");
    expect(request.headers["authorization"]).to_be("Bearer token123");
    expect(request.headers["accept"]).to_be("application/graphql-response+json");
});

test("Should handle header values with special characters", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["x-custom"] = "value-with-dashes_and_underscores.and.dots";
    expect(request.headers["x-custom"]).to_be("value-with-dashes_and_underscores.and.dots");
});

test("Should handle empty header values", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["x-empty"] = "";
    expect(request.headers["x-empty"]).to_be("");
});

test("Should handle header overwrite", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["x-value"] = "original";
    request.headers["x-value"] = "overwritten";
    expect(request.headers["x-value"]).to_be("overwritten");
});

test("Should handle router request headers with http v1", ||{
    let request = apollo_mocks::get_router_service_request();
    request.headers["x-router-header"] = "router-val";
    expect(request.headers["x-router-header"]).to_be("router-val");
});

test("Should handle router response headers with http v1", ||{
    let response = apollo_mocks::get_router_service_response();
    response.headers["x-powered-by"] = "Apollo Router v2.12.0";
    expect(response.headers["x-powered-by"]).to_be("Apollo Router v2.12.0");
});
