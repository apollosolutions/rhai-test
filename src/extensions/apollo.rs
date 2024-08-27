use apollo_router::_private::rhai as ApolloRhai;
use rhai::plugin::*;
use rhai::Engine;

pub fn register_rhai_functions(engine: &mut Engine) {
    // Don't delete this! This will likely still be used for injecting fake request/response objects!
    /*engine.register_type_with_name::<ApolloMocks>("ApolloMocks")
    .register_fn("APOLLO", ApolloMocks::new)
    .register_fn("get_supergraph_service_request", ApolloMocks::get_supergraph_service_request);

    engine.register_type_with_name::<SupergraphServiceRequestMock>("SupergraphServiceRequestMock");*/

    let mut module = exported_module!(ApolloRhai::engine::router_plugin);

    let base64_module = exported_module!(ApolloRhai::engine::router_base64);
    let json_module = exported_module!(ApolloRhai::engine::router_json);
    let sha256_module = exported_module!(ApolloRhai::engine::router_sha256);

    engine
        .register_global_module(module.into())
        .register_static_module("base64", base64_module.into())
        .register_static_module("json", json_module.into())
        .register_static_module("sha256", sha256_module.into());
}

/*
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
}*/
