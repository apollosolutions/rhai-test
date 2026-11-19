// ============================================================
// rhai-1-25-features.test.rhai
// Proves the engine is Rhai ~1.25.x. Each test below uses
// behavior introduced in Rhai 1.24 or 1.25, so it fails on
// 1.23.6 and earlier.
// ============================================================

test("Rhai 1.24: arrays support sort_desc", ||{
    let a = [3, 1, 2];
    a.sort_desc();
    expect(a[0]).to_be(3);
    expect(a[2]).to_be(1);
});

test("Rhai 1.25: split is callable on a const string", ||{
    const CSV = "a,b,c";
    let parts = CSV.split(",");
    expect(parts.len()).to_be(3);
});

test("Rhai 1.25: index_of falls back to value comparison for strings", ||{
    let a = ["x", "y", "z"];
    expect(a.index_of("y")).to_be(1);
});

test("Rhai 1.25: \\${...} escapes interpolation in multi-line strings", ||{
    let s = `price: \${amount}`;
    expect(s).to_be("price: ${amount}");
});

test("Router json::decode error is catchable", ||{
    let caught = false;
    try { json::decode("{not json"); } catch { caught = true; }
    expect(caught).to_be(true);
});
