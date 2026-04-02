# rhai-test

This is an experimental CLI tool for running unit tests against your Router rhai scripts. It allows you to write unit tests in rhai that feel familiar and natural. This provides not only the mechanism to write and run the tests but also utilities for mocking apollo objects that allow you to fully test against the Router lifecycle.

## ⚠️ Disclaimer ⚠️
This project is experimental and is not a fully-supported Apollo Graph project. We may not respond to issues and pull requests at this time.

## Rhai Version Policy

`rhai-test` tracks the version of the [Rhai scripting engine](https://rhai.rs/) that is bundled with the latest stable [Apollo Router](https://github.com/apollographql/router) release. This ensures that scripts validated by `rhai-test` behave the same way they do when running in Router.

| rhai-test version | Rhai version | Apollo Router version |
|---|---|---|
| 0.2.6 | 1.23.6 | v2.12.0 |
| 0.2.5 | 1.17.1 | — |

When a new Router release ships with an updated Rhai version, `rhai-test` should be updated to match. See the [Apollo Router release notes](https://github.com/apollographql/router/releases) for details on which Rhai version each Router release includes.

- [rhai-test](#rhai-test)
  - [⚠️ Disclaimer ⚠️](#️-disclaimer-️)
  - [Rhai Version Policy](#rhai-version-policy)
  - [Example](#example)
  - [Getting Started](#getting-started)
    - [Config File](#config-file)
    - [Writing your first test](#writing-your-first-test)
    - [Running your tests](#running-your-tests)
    - [Watch Mode](#watch-mode)
  - [Router Rhai Functions](#router-rhai-functions)
  - [Mocks](#mocks)
    - [Lifecycle Methods](#lifecycle-methods)
    - [Logging Methods](#logging-methods)
    - [`set_env`](#set_env)
  - [Expector](#expector)
    - [`to_be(String)`](#to_bestring)
    - [`to_match(String)`](#to_matchstring)
    - [`to_exist()`](#to_exist)
    - [`not()`](#not)
    - [`to_throw()`](#to_throw)
    - [`to_throw_message(String)`](#to_throw_messagestring)
    - [`to_throw_status(Int)`](#to_throw_statusint)
    - [`to_throw_status_and_message(Int, String)`](#to_throw_status_and_messageint-string)
    - [`to_log()`](#to_log)
    - [`to_log_message(String)`](#to_log_messagestring)
  - [Recipes](#recipes)
    - [Checking for error logging when a function throws an error](#checking-for-error-logging-when-a-function-throws-an-error)
    - [Testing against subgraph request](#testing-against-subgraph-request)

## Getting Started

### Install from GitHub Releases

The installer downloads a published **GitHub Release** asset. The script on `main` embeds a `PACKAGE_VERSION` (for example `v0.2.5`); that tag must exist on [Releases](https://github.com/apollosolutions/rhai-test/releases) or the download will fail.

```sh
curl -sSL https://raw.githubusercontent.com/apollosolutions/rhai-test/refs/heads/main/installers/nix/install.sh | sh
```

This installs the binary under `~/.rhai-test/bin` and tries to append that directory to your `PATH` (for example in `~/.zshrc` / `~/.bashrc`). Open a **new terminal** (or `exec "$SHELL" -l`) so `PATH` picks up the change.

If `rhai-test` is not found, add it manually:

```sh
export PATH="$PATH:$HOME/.rhai-test/bin"
```

Confirm the CLI is on your `PATH`:

```sh
rhai-test --version
```

To install a **specific** published version, set `VERSION` to a release tag (for example `v0.2.5`):

```sh
curl -sSL https://raw.githubusercontent.com/apollosolutions/rhai-test/refs/heads/main/installers/nix/install.sh | VERSION="v0.2.5" sh
```

The installer does **not** build from a git branch. For unreleased changes, build [from source](#building-from-source-unreleased--branch-changes) instead.

### Run your first tests

`rhai-test` expects a config file in the current directory (by default `rhai-test.config.json`). From a project that contains your Rhai sources and tests:

1. Add a config file (see [Config File](#config-file) for all options). Minimal example:

```json
{
  "testMatch": ["**/*.test.rhai"],
  "basePath": "."
}
```

2. Run the test runner:

```sh
rhai-test
```

If you cloned this repository and want to run its example suite, point `basePath` at `examples` (see the [example config](#config-file) below).

### Building from source (unreleased / branch changes)

Use this when you need a build that is not published as a GitHub Release yet.

Prerequisites:

- **Rust toolchain (Cargo)** — install via [rustup](https://rustup.rs/):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

- **Git** — to clone and checkout a branch.

```sh
git clone https://github.com/apollosolutions/rhai-test.git
cd rhai-test
git checkout <your-branch>   # optional
cargo build --release
./target/release/rhai-test
```

### Uninstall

To uninstall `rhai-test`, remove the install directory and undo the PATH changes made by the installer.

```sh
rm -rf ~/.rhai-test
```

The installer appends `~/.rhai-test/bin` to your shell config. Remove the line below from `~/.zshrc` and/or `~/.bashrc` if present:

```sh
export PATH="$PATH:~/.rhai-test/bin"
```

Then restart your shell:

```sh
exec zsh
```

Verify it is uninstalled:

```sh
command -v rhai-test || echo "rhai-test not found"
```

### Releasing (maintainers)

Merging a pull request into `main` **does not** automatically publish a GitHub Release or upload binaries. When you are ready to ship:

1. Use repository automation: open **Actions → Build and Release**, then **Run workflow** (`workflow_dispatch`). That pipeline runs tests, bumps the version with [Knope](https://knope.dev/), builds archives for Linux (x86_64), macOS (Apple Silicon), and Windows, creates the GitHub Release with those assets, and commits an update to `installers/nix/install.sh` so `PACKAGE_VERSION` matches the new tag.
2. After it finishes, confirm the new tag appears under [Releases](https://github.com/apollosolutions/rhai-test/releases) and that the default [install steps](#install-from-github-releases) work on your machine.

Until a release exists for the version pinned in `install.sh` on `main`, the default one-liner install will fail at the download step—run the release workflow after merging version bumps, or install from source.

## Example

Given this rhai script:

```rhai
fn process_request(request) {
    log_info("processing request");
    let valid_client_names = ["apollo-client", "retail-website"];

    if ("apollographql-client-version" in request.headers && "apollographql-client-name" in request.headers) {
      let client_header = request.headers["apollographql-client-version"];
      let name_header = request.headers["apollographql-client-name"];      
    
      if !valid_client_names.contains(name_header) {
        log_error("Invalid client name provided");
        throw #{
          status: 401,
          message: "Invalid client name provided"
        };
      }
  
      if client_header == "" {
        log_error("No client version provided");
        throw #{
          status: 401,
          message: "No client version provided"
        };
      }
    }
    else {
      log_error("No client headers set. Please provide headers: apollographql-client-name and apollographql-client-version");
      throw #{
        status: 401,
        message: "No client headers set. Please provide headers: apollographql-client-name and apollographql-client-version"
      };
    }    
}
```

Here is a set of unit tests:

```rhai

test("Should throw an error when no client headers are provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);};

    expect(execute).to_throw();
});

test("Should throw an error with message when no client headers are provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);};

    expect(execute).to_throw_message("No client headers set. Please provide headers: apollographql-client-name and apollographql-client-version");
});

test("Should throw an error when apollographql-client-version header is not provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    expect(execute).to_throw_message("No client headers set.");
});

test("Should throw an error when apollographql-client-name header is not provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-version"] = "1.0";

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    expect(execute).to_throw_status(401);
});

test("Should throw an error when client header is invalid", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "abc123";
    request.headers["apollographql-client-version"] = "1.0";

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    expect(execute).to_throw_status_and_message(401, "Invalid client name provided");
});

test("Should throw an error when client version header is blank", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";
    request.headers["apollographql-client-version"] = "";

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    expect(execute).to_throw_status_and_message(401, "No client version provided");
});

test("Should not throw an error when clients header are provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";
    request.headers["apollographql-client-version"] = "1.0";

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    expect(execute).not().to_throw();
});
```

You can find more examples in the examples directory.

### Config File

To run the CLI, you will need a `rhai-test.config.json` config file. If you prefer a different name, you can specify this with the `--config` arg when calling the cli.

Config values:

| Name | Default | required | Description |
|-----|-----|-----|-----|
| testMatch | - | Yes | An array of glob patterns of where to find test files. Recommended value: `["**/*.test.rhai"]`
| basePath | - | Yes | Where your rhai files are located |
| coverage | false | no | [EXPERIMENTAL] Whether or not to provide a coverage report. Note these is very experimental and should not be relied on for accurate metrics at this time. |

Example config file:

```json
{
  "testMatch": ["**/*.test.rhai"],
  "basePath": "examples",
  "coverage": false
}
```

### Writing your first test

The most basic test you can write has an expect statement that assets something to be true. This test should be added to a test file (E.g. `my_first_test.test.rhai`).

```rhai
test("This is my first test", ||{
    expect("a").to_be("a");
});
```

### Running your tests

To run your tests, simply run the CLI.

```sh
rhai-test
```

### Watch Mode

You can pass a `--watch` flag to have the CLI watch for changes to your rhai files and re-run the tests every time it detects a change

```sh
rhai-test --watch
```

## Router Rhai Functions

Note that all Router Rhai functions are injected in and can be used directly in your tests:

```rhai
test("Should generate a uuid", ||{
    let uuid = uuid_v4();

    expect(uuid).to_match(".{8}-.{4}-.{4}-.{4}-.{12}");
});
```

## Mocks

### Lifecycle Methods

You can get a mock of each of the request/response objects in each of the parts of the Router lifecycle by calling `apollo_mocks`.

```rhai
let router_request = apollo_mocks::get_router_service_request();
let router_response = apollo_mocks::get_router_service_response();
let supergraph_request = apollo_mocks::get_supergraph_service_request();
let supergraph_response = apollo_mocks::get_supergraph_service_response();
let execution_request = apollo_mocks::get_execution_service_request();
let execution_response = apollo_mocks::get_execution_service_response();
// Note that you need to pass a supergraph_request to create a subgraph_request
let subgraph_request = apollo_mocks::get_subgraph_service_request(supergraph_request);
let subgraph_response = apollo_mocks::get_subgraph_service_response();
```

You can then set values on these and pass them into your functions.

```
test("Should not throw an error when clients header are provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";
    request.headers["apollographql-client-version"] = "1.0";

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    expect(execute).not().to_throw();
});
```

### Logging Methods

This library injects in identifiers for each of the Router logging methods. This can be used to test that a particular log method was called after calling your functions.

You can either use `to_log` to simply check for a log of that level or `to_log_message` to check for a specific message of that level.

```rhai
test("Should log processing request when process_request is called", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";
    request.headers["apollographql-client-version"] = "1.0";

    let execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    execute.call();

    expect(log_info).to_log();
});

test("Should log processing request when process_request is called", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";
    request.headers["apollographql-client-version"] = "1.0";

    let execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    execute.call();

    expect(log_info).to_log_message("processing request");
});
```

The following logging methods can be checked:

- `log_trace`
- `log_debug`
- `log_info`
- `log_warn`
- `log_error`

### `set_env`
If you need to set an environment variable so you can pull it out of a script, you can do so with `test_helpers::set_env`:

```rhai
test("Should get environment variables", ||{
    test_helpers::set_env("MY_COOL_ENV_VAR", "hello");
    let result = `${env::get("MY_COOL_ENV_VAR")}`;

    expect(result).to_be("hello");
});
```

## Expector

When writing a test, it should contain one or more expect statements.

There are a handful of methods you can fun against an `expect` statement.

### `to_be(String)`

Checks if two values are equal.

```rhai
test("Should encode text to base64", ||{
    let original = "alice and bob";
    let encoded = base64::encode(original);

    expect(encoded).to_be("YWxpY2UgYW5kIGJvYg==");
});
```

### `to_match(String)`

Checks if a value matches a regular expression.

```rhai
test("Should generate a uuid", ||{
    let uuid = uuid_v4();

    expect(uuid).to_match(".{8}-.{4}-.{4}-.{4}-.{12}");
});
```

### `to_exist()`

Checks if a value exists

```rhai
test("Should encode text to base64", ||{
    expect("a").to_exist();
});
```

### `not()`

You can inverse your expector to write "not" logic:

```rhai
test("Should pass a negative string assert", ||{
    expect("a").not().to_be("b")
});
```

### `to_throw()`

Runs a provided method and checks if it throws an error.

```rhai
test("Should throw an error when no client headers are provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);};

    expect(execute).to_throw();
});
```

### `to_throw_message(String)`

Runs a provided method and checks if it throws an error with a specific message, matched with a regular expression.

```rhai
test("Should throw an error with message when no client headers are provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);};

    expect(execute).to_throw_message("No client headers set. Please provide headers: apollographql-client-name and apollographql-client-version");
});
```

### `to_throw_status(Int)`

Runs a provided method and checks if it throws an error with a specific status code.

```rhai
test("Should throw an error when apollographql-client-name header is not provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-version"] = "1.0";

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    expect(execute).to_throw_status(401);
});
```

### `to_throw_status_and_message(Int, String)`

Runs a provided method and checks if it throws an error with a specific message, matched with a regular expression, and a specific status code.

```rhai
test("Should throw an error when client header is invalid", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "abc123";
    request.headers["apollographql-client-version"] = "1.0";

    const execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    expect(execute).to_throw_status_and_message(401, "Invalid client name provided");
});
```

### `to_log()`

Checks if a particular logging method was called.

```rhai
test("Should log processing request when process_request is called", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";
    request.headers["apollographql-client-version"] = "1.0";

    let execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    execute.call();

    expect(log_info).to_log();
});
```

### `to_log_message(String)`

Checks if a particular logging method was called with a message, matched against a regular expression.

```rhai
test("Should log processing request when process_request is called", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";
    request.headers["apollographql-client-version"] = "1.0";

    let execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    execute.call();

    expect(log_info).to_log_message("processing request");
});
```

## Recipes

### Checking for error logging when a function throws an error

If you have designed a test in a way that results in a function call throwing an error, you will likely need to wrap the method call in a try/catch to "bury" the error so that you can check if the log method was called.

```rhai
test("Should log an error when version header is not provided", ||{
    let request = apollo_mocks::get_supergraph_service_request();
    request.headers["apollographql-client-name"] = "apollo-client";

    let execute = || {
        import "client_id" as client_id;
        client_id::process_request(request);
    };

    try {execute.call();} catch {}

    expect(log_error).to_log_message("No client headers set");
});
```

### Testing against subgraph request

In order to create a subgraph request mock, you will need to create a supergraph request mock. This will allow you to modify headers for testing these types of requests. If you try to modify the headers on a `subgraph_request`, you will receive an error.

```
test("Should be able to modify subgraph requestsvia supergraph request", ||{
    let supergraph_request = apollo_mocks::get_supergraph_service_request();
    supergraph_request.headers["assetid"] = "abc123";
    let subgraph_request = apollo_mocks::get_subgraph_service_request(supergraph_request);

    import "headers" as headers;
    headers::rename_header(subgraph_request);

    expect(subgraph_request.subgraph.headers["original_assetid"]).to_be("abc123");
});
```

