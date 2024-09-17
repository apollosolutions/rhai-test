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

test("Should be able to compare ints", ||{
    expect(1).to_be(1);
});

test("Should be able to negative compare ints", ||{
    expect(1).not().to_be(2);
});

test("Should be able to compare nothing", ||{
    expect(()).to_be(());
});

test("Should be able to check that something exists", ||{
    expect("a").to_exist();
});

test("Should be able to check that something does not exist", ||{
    expect(()).not().to_exist();
});