test("Should encode text to base64", ||{
    let original = "alice and bob";
    let encoded = base64::encode(original);

    expect(encoded).to_be("YWxpY2UgYW5kIGJvYg==");
});

test("Should decode json string to object", ||{
    let original = `{"valid":"wee"}`;
    let decoded = json::decode(original);

    expect(decoded.valid).to_be("wee");
});

test("Should provide a sha256 hash of a string", ||{
    let sha = sha256::digest("hello world");

    expect(sha).to_be("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
});

test("Should generate a uuid", ||{
    let uuid = uuid_v4();

    expect(uuid).to_match(".{8}-.{4}-.{4}-.{4}-.{12}");
});

test("Should get environment variables", ||{
    get_testing_utils().set_env("MY_COOL_ENV_VAR", "hello");
    let result = `${env::get("MY_COOL_ENV_VAR")}`;

    expect(result).to_be("hello");
});

test("Should do stuff", ||{
    let service = apollo_mocks::get_supergraph_service();
    let request = apollo_mocks::get_supergraph_service_request();

    let request_callback = |request| {
        print("Supergraph service: Client request received");
    };

    service.map_request(request_callback);

    print(request.body.query);

    expect("a").to_be("a");
});