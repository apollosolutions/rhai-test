use rhai::Engine;

pub fn register_rhai_functions(engine: &mut Engine){
    engine.register_type_with_name::<ApolloMocks>("ApolloMocks")
    .register_fn("APOLLO", ApolloMocks::new)
    .register_fn("get_supergraph_service_request", ApolloMocks::get_supergraph_service_request);

    engine.register_type_with_name::<SupergraphServiceRequestMock>("SupergraphServiceRequestMock");
}

#[derive(Debug, Clone)]
pub struct SupergraphServiceRequestMock {
    pub headers: http::HeaderMap
}

impl SupergraphServiceRequestMock {
    pub fn new() -> Self {
        Self {
            headers: http::HeaderMap::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApolloMocks {

}

impl ApolloMocks {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_supergraph_service_request(&mut self) -> SupergraphServiceRequestMock {
        SupergraphServiceRequestMock::new()
    }
}