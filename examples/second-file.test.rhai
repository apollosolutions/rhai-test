test("this is a passing test", ||{
    expect(1).to_be(1);
    expect("c").to_be("c");
});


/*
self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
*/


/*
self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Ok(()));
*/