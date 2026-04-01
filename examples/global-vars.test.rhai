// ============================================================
// global-vars.test.rhai
// Tests Router global variables exposed via on_var callback.
// Ensures registration::register(engine) didn't break the
// variable interception for Router.* constants.
// ============================================================

test("Should have Router.APOLLO_SDL available", ||{
    let sdl = Router.APOLLO_SDL;
    expect(sdl).to_exist();
});

test("Should have Router.APOLLO_START available", ||{
    let start = Router.APOLLO_START;
    expect(start).to_exist();
});

test("Should have Router.APOLLO_AUTHENTICATION_JWT_CLAIMS available", ||{
    let claims_key = Router.APOLLO_AUTHENTICATION_JWT_CLAIMS;
    expect(claims_key).to_be("apollo_authentication::JWT::claims");
});

test("Should have Router.APOLLO_ENTITY_CACHE_KEY available", ||{
    let cache_key = Router.APOLLO_ENTITY_CACHE_KEY;
    expect(cache_key).to_be("apollo_entity_cache::key");
});

test("Should have Router.APOLLO_OPERATION_ID available", ||{
    let op_id = Router.APOLLO_OPERATION_ID;
    expect(op_id).to_be("apollo_operation_id");
});

test("Should have Router.APOLLO_SUBSCRIPTION_WS_CUSTOM_CONNECTION_PARAMS available", ||{
    let sub_key = Router.APOLLO_SUBSCRIPTION_WS_CUSTOM_CONNECTION_PARAMS;
    expect(sub_key).to_be("apollo.subscription.custom_connection_params");
});
