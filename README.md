> This is currently non-functional without using a local router build. Please talk to Andrew McGivery if you want to try this out or just get a demo!

# rhai-test

1. Pull down router branch: https://github.com/andrewmcgivery/router/tree/feature/rhaitest
2. Update `apollo-router` in `cargo.toml` of this repo to the path of your local copy of `/router/apollo-router`
3. Run `cargo run` to execute all `*.test.rhai` files with their tests.

![alt text](screenshot.png)

> Note: `cargo run` will take a bit of time the first time because it needs to build the Router lib first before building this. However, it will be fast after that as it will use a cached build.