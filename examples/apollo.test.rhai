test("this is a test of using apollo plugin", ||{
    let request = APOLLO().get_supergraph_service_request();

    expect("a").to_be("a");
});