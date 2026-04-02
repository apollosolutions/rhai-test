// ============================================================
// rhai-engine-functions.test.rhai
// Tests that all the Router-provided rhai engine functions
// (base64, json, sha256, uuid, env) still work after the
// rhai 1.17.1 → 1.23.6 bump. This overlaps with apollo.test.rhai
// but adds edge cases.
// ============================================================

test("Should base64 encode empty string", ||{
    let encoded = base64::encode("");
    expect(encoded).to_be("");
});

test("Should base64 encode and decode round-trip", ||{
    let original = "Hello, Apollo Router!";
    let encoded = base64::encode(original);
    let decoded = base64::decode(encoded);
    expect(decoded).to_be(original);
});

test("Should base64 encode unicode characters", ||{
    let original = "café";
    let encoded = base64::encode(original);
    // Just verify it doesn't crash and produces output
    expect(encoded).to_exist();
});

test("Should json decode nested objects", ||{
    let json_str = `{"outer":{"inner":"value","count":42}}`;
    let decoded = json::decode(json_str);
    expect(decoded.outer.inner).to_be("value");
});

test("Should json decode arrays", ||{
    let json_str = `{"items":["a","b","c"]}`;
    let decoded = json::decode(json_str);
    expect(decoded.items[0]).to_be("a");
    expect(decoded.items[2]).to_be("c");
});

test("Should sha256 hash empty string", ||{
    let hash = sha256::digest("");
    expect(hash).to_be("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
});

test("Should generate unique UUIDs", ||{
    let uuid1 = uuid_v4();
    let uuid2 = uuid_v4();
    // They should both match UUID format
    expect(uuid1).to_match(".{8}-.{4}-.{4}-.{4}-.{12}");
    expect(uuid2).to_match(".{8}-.{4}-.{4}-.{4}-.{12}");
    // And they should be different
    expect(uuid1).not().to_be(uuid2);
});

test("Should set and get environment variables", ||{
    test_helpers::set_env("PR14_TEST_VAR", "rhai-1.23.6");
    let result = `${env::get("PR14_TEST_VAR")}`;
    expect(result).to_be("rhai-1.23.6");
});
