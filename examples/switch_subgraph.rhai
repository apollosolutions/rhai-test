fn subgraph_service(service, subgraph) {
    if subgraph == "my_awesome_subgraph" {
        const request_callback = Fn("switch_subgraph_url");
        service.map_request(request_callback);
    }
}

fn switch_subgraph_url(request){
    let operations_to_send_to_new_implementation = ["operation1", "operation2"];

    if operations_to_send_to_new_implementation.contains(request.body.operation_name) {
        request.subgraph.uri.path = "/my-api";
    }
}