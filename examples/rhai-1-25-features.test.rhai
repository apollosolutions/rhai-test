// ============================================================
// rhai-1-25-features.test.rhai
// The first four tests exercise behavior added in Rhai 1.24/1.25
// (sort_desc, split on const strings, index_of string fallback,
// and the `\${...}` escape). The last test is a regression guard
// for Router's json::decode error handling.
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
