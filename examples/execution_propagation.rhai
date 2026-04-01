// Customer rhai script: execution_propagation.rhai
// Propagates a tenant header from the incoming request into
// the execution context so subgraphs can use it downstream.

fn execution_request(request) {
    if "x-tenant-id" in request.headers {
        let tenant = request.headers["x-tenant-id"];

        if tenant == "" {
            log_error("Tenant ID header is blank");
            throw #{
                status: 400,
                message: "x-tenant-id header must not be blank"
            };
        }

        log_info(`Propagating tenant: ${tenant}`);
    } else {
        log_warn("No x-tenant-id header present");
    }
}
