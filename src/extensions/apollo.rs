use crate::engine::logging_container::{LogLevel, LoggingContainer};
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
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Attach all the Router-specific functions to the Engine
pub fn register_rhai_functions_and_types(
    engine: &mut Engine,
    logging_container: Arc<Mutex<LoggingContainer>>,
) {
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
            print!("{}", message);
        });

    // Register logging functions for capturing logs so we can write tests against them
    let logging_container_clone = logging_container.clone();
    engine.register_fn("log_trace", move |message: Dynamic| {
        logging_container_clone
            .lock()
            .unwrap()
            .add_log(message.to_string(), LogLevel::TRACE);
    });

    let logging_container_clone = logging_container.clone();
    engine.register_fn("log_debug", move |message: Dynamic| {
        logging_container_clone
            .lock()
            .unwrap()
            .add_log(message.to_string(), LogLevel::DEBUG);
    });

    let logging_container_clone = logging_container.clone();
    engine.register_fn("log_info", move |message: Dynamic| {
        logging_container_clone
            .lock()
            .unwrap()
            .add_log(message.to_string(), LogLevel::INFO);
    });

    let logging_container_clone = logging_container.clone();
    engine.register_fn("log_warn", move |message: Dynamic| {
        logging_container_clone
            .lock()
            .unwrap()
            .add_log(message.to_string(), LogLevel::WARN);
    });

    let logging_container_clone = logging_container.clone();
    engine.register_fn("log_error", move |message: Dynamic| {
        logging_container_clone
            .lock()
            .unwrap()
            .add_log(message.to_string(), LogLevel::ERROR);
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
            // Intercept references to logging methods as a variable so we can write tests to see if they were called
            "log_trace" => Ok(Some(rhai::Dynamic::from(LogLevel::TRACE))),
            "log_debug" => Ok(Some(rhai::Dynamic::from(LogLevel::DEBUG))),
            "log_info" => Ok(Some(rhai::Dynamic::from(LogLevel::INFO))),
            "log_warn" => Ok(Some(rhai::Dynamic::from(LogLevel::WARN))),
            "log_error" => Ok(Some(rhai::Dynamic::from(LogLevel::ERROR))),
            // Return Ok(None) to continue with the normal variable resolution process.
            _ => Ok(None),
        }
    });
}

/// Register our apollo_mocks interface
pub fn register_mocking_functions(engine: &mut Engine) {
    engine
        .register_type_with_name::<apollo_mocks::SupergraphService>("SupergraphService")
        .register_fn("map_request", apollo_mocks::SupergraphService::map_request)
        .register_fn(
            "has_mapped_request",
            apollo_mocks::SupergraphService::has_mapped_request,
        );

    let apollo_mocks_module = exported_module!(apollo_mocks);

    engine.register_static_module("apollo_mocks", apollo_mocks_module.into());
}

#[export_module]
mod apollo_mocks {
    use apollo_router::_private::rhai::execution;
    use apollo_router::_private::rhai::router;
    use apollo_router::_private::rhai::subgraph;
    use apollo_router::_private::rhai::supergraph;
    use std::sync::Mutex;

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

        pub fn has_mapped_request(&mut self) -> bool {
            return self.request_callback.is_some();
        }
    }

    #[rhai_fn()]
    pub(crate) fn get_supergraph_service() -> SupergraphService {
        SupergraphService::new()
    }

    #[rhai_fn()]
    pub(crate) fn get_router_service_request(
    ) -> Shared<Mutex<std::option::Option<apollo_router::services::router::Request>>> {
        let request = router::Request::fake_builder()
            .method(Method::POST)
            .context(Context::new())
            .build()
            .unwrap();
        let shared_request = Arc::new(Mutex::new(Some(request)));
        shared_request
    }

    #[rhai_fn()]
    pub(crate) fn get_router_service_response(
    ) -> Shared<Mutex<std::option::Option<apollo_router::services::router::Response>>> {
        let response = router::Response::fake_builder()
            .context(Context::new())
            .build()
            .unwrap();
        let shared_response = Arc::new(Mutex::new(Some(response)));
        shared_response
    }

    #[rhai_fn()]
    pub(crate) fn get_supergraph_service_request(
    ) -> Shared<Mutex<std::option::Option<apollo_router::services::supergraph::Request>>> {
        let request = supergraph::Request::fake_builder()
            .method(Method::POST)
            .context(Context::new())
            .build()
            .unwrap();
        let shared_request = Arc::new(Mutex::new(Some(request)));
        shared_request
    }

    #[rhai_fn()]
    pub(crate) fn get_supergraph_service_response(
    ) -> Shared<Mutex<std::option::Option<apollo_router::services::supergraph::Response>>> {
        let response = supergraph::Response::fake_builder()
            .context(Context::new())
            .build()
            .unwrap();
        let shared_response = Arc::new(Mutex::new(Some(response)));
        shared_response
    }

    #[rhai_fn()]
    pub(crate) fn get_execution_service_request(
    ) -> Shared<Mutex<std::option::Option<apollo_router::services::execution::Request>>> {
        let request = execution::Request::fake_builder()
            .context(Context::new())
            .build();
        let shared_request = Arc::new(Mutex::new(Some(request)));
        shared_request
    }

    #[rhai_fn()]
    pub(crate) fn get_execution_service_response(
    ) -> Shared<Mutex<std::option::Option<apollo_router::services::execution::Response>>> {
        let response = execution::Response::fake_builder()
            .context(Context::new())
            .build()
            .unwrap();
        let shared_response = Arc::new(Mutex::new(Some(response)));
        shared_response
    }

    #[rhai_fn()]
    pub(crate) fn get_subgraph_service_request(
        supergraph_request: Arc<Mutex<Option<apollo_router::services::supergraph::Request>>>,
    ) -> Shared<Mutex<std::option::Option<apollo_router::services::subgraph::Request>>> {
        let request_guard = supergraph_request.lock().unwrap();
        let raw_supergraph_request = &request_guard.as_ref().unwrap().supergraph_request;
        let mut new_supergraph_request = http::Request::builder()
            .uri(raw_supergraph_request.uri().clone())
            .method(raw_supergraph_request.method().clone())
            .body(raw_supergraph_request.body().clone())
            .unwrap();
        *new_supergraph_request.headers_mut() = raw_supergraph_request.headers().clone();

        let request = subgraph::Request::fake_builder()
            .context(Context::new())
            .supergraph_request(Arc::new(new_supergraph_request))
            .build();
        let shared_request = Arc::new(Mutex::new(Some(request)));
        shared_request
    }

    #[rhai_fn()]
    pub(crate) fn get_subgraph_service_response(
    ) -> Shared<Mutex<std::option::Option<apollo_router::services::subgraph::Response>>> {
        let response = subgraph::Response::fake_builder()
            .context(Context::new())
            .build();
        let shared_response = Arc::new(Mutex::new(Some(response)));
        shared_response
    }
}
