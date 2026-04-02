// Customer rhai script: router_tracing.rhai
// Injects a request ID on every incoming router request,
// and echoes it back on the response for client-side tracing.

fn router_request(request) {
    // If the client didn't provide a trace ID, generate one
    if !("x-request-id" in request.headers) {
        request.headers["x-request-id"] = `generated-${uuid_v4()}`;
        log_info("Generated new request ID");
    } else {
        log_info("Using client-provided request ID");
    }
}

fn router_response(response) {
    // Always echo the request ID back so clients can correlate
    if "x-request-id" in response.headers {
        log_debug("Request ID already on response");
    } else {
        response.headers["x-request-id"] = "unknown";
        log_warn("No request ID found on response context");
    }
}
