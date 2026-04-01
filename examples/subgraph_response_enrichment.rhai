// Customer rhai script: subgraph_response_enrichment.rhai
// Tags subgraph responses with a processing marker,
// and adds a served-by header on the supergraph response.

fn subgraph_response(response) {
    response.headers["x-subgraph-processed"] = "true";
    log_debug("Marked subgraph response as processed");
}

fn supergraph_response(response) {
    response.headers["x-served-by"] = "apollo-router";
    log_info("Tagged supergraph response with served-by header");
}
