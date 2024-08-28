use std::sync::Arc;
use std::time::Instant;

use apollo_router::_private::rhai as ApolloRhai;
use apollo_router::_private::rhai::engine::SharedMut;
use apollo_router::_private::rhai::{execution, router, subgraph, supergraph};
use apollo_router::if_subgraph;
use apollo_router::register_rhai_interface;
use apollo_router::register_rhai_router_interface;
use apollo_router::Context;
use apollo_router::_private::rhai::engine::OptionDance;
use apollo_router::graphql::Request;
use http::HeaderMap;
use http::Method;
use http::StatusCode;
use http::Uri;
use rhai::Shared;
use rhai::{plugin::*, Map};
use rhai::{Engine, FnPtr};
use serde_json::json;

pub fn register_rhai_functions_and_types(engine: &mut Engine) {
    let mut module = exported_module!(ApolloRhai::engine::router_plugin);
    combine_with_exported_module!(&mut module, "header", ApolloRhai::engine::router_header_map);
    combine_with_exported_module!(&mut module, "method", ApolloRhai::engine::router_method);
    combine_with_exported_module!(&mut module, "status_code", ApolloRhai::engine::status_code);
    combine_with_exported_module!(&mut module, "context", ApolloRhai::engine::router_context);

    let base64_module = exported_module!(ApolloRhai::engine::router_base64);
    let json_module = exported_module!(ApolloRhai::engine::router_json);
    let sha256_module = exported_module!(ApolloRhai::engine::router_sha256);

    let expansion_module = exported_module!(ApolloRhai::engine::router_expansion);

    engine
        .register_global_module(module.into())
        .register_static_module("base64", base64_module.into())
        .register_static_module("json", json_module.into())
        .register_static_module("sha256", sha256_module.into())
        .register_static_module("env", expansion_module.into())
        .register_iterator::<HeaderMap>()
        .on_print(move |message| {
            // TODO: Should we conditionally output based on a configured log level?
            // TODO: Can we collect all the print-type methods and spit them out similar to how Jest does it?
            print!("{}", message);
        })
        .register_fn("log_trace", move |message: Dynamic| {
            // TODO: Should we conditionally output based on a configured log level?
        })
        .register_fn("log_debug", move |message: Dynamic| {
            // TODO: Should we conditionally output based on a configured log level?
        })
        .register_fn("log_info", move |message: Dynamic| {
            // TODO: Should we conditionally output based on a configured log level?
        })
        .register_fn("log_warn", move |message: Dynamic| {
            // TODO: Should we conditionally output based on a configured log level?
        })
        .register_fn("log_error", move |message: Dynamic| {
            // TODO: Should we conditionally output based on a configured log level?
        });

    register_rhai_router_interface!(engine, router);
    register_rhai_interface!(engine, supergraph, execution, subgraph);

    let mut global_variables = Map::new();
    global_variables.insert("APOLLO_SDL".into(), "".to_string().into()); // TODO: Allow SDL to be inserted via helper methods?
    global_variables.insert("APOLLO_START".into(), Instant::now().into());
    global_variables.insert(
        "APOLLO_AUTHENTICATION_JWT_CLAIMS".into(),
        "apollo_authentication::JWT::claims".to_string().into(), // TODO: Pull this from the proper constant from Router
    );
    global_variables.insert(
        "APOLLO_SUBSCRIPTION_WS_CUSTOM_CONNECTION_PARAMS".into(),
        "apollo.subscription.custom_connection_params"
            .to_string()
            .into(), // TODO: Pull this from the proper constant from Router
    );
    global_variables.insert(
        "APOLLO_ENTITY_CACHE_KEY".into(),
        "apollo_entity_cache::key".into(),
    ); // TODO: Pull this from the proper constant from Router
    global_variables.insert("APOLLO_OPERATION_ID".into(), "apollo_operation_id".into()); // TODO: Pull this from the proper constant from Router

    let shared_globals = Arc::new(global_variables);

    #[allow(deprecated)]
    engine.on_var(move |name, _index, _context| {
        match name {
            // Intercept attempts to find "Router" variables and return our "global variables"
            // Note: Wrapped in an Arc to lighten the load of cloning.
            "Router" => Ok(Some((*shared_globals).clone().into())),
            // Return Ok(None) to continue with the normal variable resolution process.
            _ => Ok(None),
        }
    });
}

pub fn register_mocking_functions(engine: &mut Engine) {
    engine
        .register_type_with_name::<apollo_mocks::SupergraphService>("SupergraphService")
        .register_fn("map_request", apollo_mocks::SupergraphService::map_request);

    let apollo_mocks_module = exported_module!(apollo_mocks);

    engine.register_static_module("apollo_mocks", apollo_mocks_module.into());
}

#[export_module]
mod apollo_mocks {
    use std::sync::Mutex;

    use apollo_router::_private::rhai::{execution, router, subgraph, supergraph};

    #[derive(Debug, Clone)]
    pub struct SupergraphService {
        request_callback: Option<FnPtr>,
    }

    impl SupergraphService {
        pub fn new() -> Self {
            Self {
                request_callback: None,
            }
        }

        pub fn map_request(&mut self, func: FnPtr) {
            self.request_callback = Some(func);
        }
    }

    #[rhai_fn()]
    pub(crate) fn get_supergraph_service() -> SupergraphService {
        SupergraphService::new()
    }

    #[rhai_fn()]
    pub(crate) fn get_supergraph_service_request(
    ) -> Shared<Mutex<std::option::Option<apollo_router::services::supergraph::Request>>> {
        let request = supergraph::Request::builder()
            .header("a", "b")
            .header("a", "c")
            .uri(Uri::from_static("http://example.com"))
            .method(Method::POST)
            .query("query { topProducts }")
            .operation_name("Default")
            .context(Context::new())
            .extension("foo", json!({}))
            .variable("bar", json!({}))
            .build()
            .unwrap();
        let shared_request = Arc::new(Mutex::new(Some(request)));
        shared_request
    }
}
