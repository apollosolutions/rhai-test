test("this is a passing test", ||{
    expect("a").to_be("a")
});

test("this is a failing test", ||{
    expect("a").to_be("b")
});