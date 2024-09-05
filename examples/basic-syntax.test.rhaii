test("Should pass a positive string assert", ||{
    expect("a").to_be("a")
});

test("Should pass a negative string assert", ||{
    expect("a").not().to_be("b")
});

test("Should pass a positive match assert", ||{
    expect("a-b-c").to_match(".-.-.")
});

test("Should pass a negative match assert", ||{
    expect("304593459345w45645").not().to_match(".-.-.")
});