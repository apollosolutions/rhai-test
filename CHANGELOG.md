## Unreleased

### 🚀 Features

- Subgraph response mocks now expose a writable `subgraph_request_id`, enabling tests that exercise the request/response id correlation pattern. ([AS-389](https://apollographql.atlassian.net/browse/AS-389), requested via [TSH-22538](https://apollographql.atlassian.net/browse/TSH-22538))
- `request.context` and `response.context` now support `remove(key)`, returning the removed value (or unit if the key was absent). Keeps parity with the existing `insert` / `upsert` / indexer-get surface.

### 🛠 Maintenance

- Bumped `apollo-router` git pin to include the two additions above. Router fork branch `feature/rhaitest-v2.12.0` remains the source; pin switched from `branch = ...` to `rev = ...` for reproducibility.

## 0.3.0 (2026-04-09)

### ❗️Breaking ❗

#### rhai engine updated from 1.17.1 to 1.23.6. Scripts using

deprecated or removed rhai APIs may need to be updated.

### 🚀 Features

- bump rhai dependency to 1.23.6 to support Apollo Router v2.12.0
- update rhai to 1.23.6 for Apollo Router v2.12.0 compatibility

## 0.2.5 (2024-12-06)

### 🚀 Features

- Added support for URI type to be passed to expect function

### 🐛 Fixes

- bug: CLI was returning a 0 exit code on test failures
- bug: test suites were not being counted if they had a syntax error

## 0.2.4 (2024-11-05)

### 🚀 Features

#### Fix install script issue on linux

Fixed issue in install script that would result in "has_required_glibc: command not found" error and then it attempting to install linux-musl which does not exist.

## 0.2.3 (2024-10-01)

### 🚀 Features

- Added --watch flag to run tester in file watch mode

## 0.2.2 (2024-09-30)

### 🚀 Features

- Sign binary so it can be easily run on Apple machines

## 0.2.1 (2024-09-18)

### 🚀 Features

- Tests can now contain more than one expect statement

## 0.2.0 (2024-09-17)

### ❗️Breaking ❗

- Modify get_subgraph_service_request to require a supergraph_request so that headers can be set on those kinds of tests

### 🚀 Features

- Add support for int and () types for expectors
- Add to_exist() expecter function

## 0.1.2 (2024-09-12)

### 🐛 Fixes

- Fix release to include all artifacts

## 0.1.1 (2024-09-12)

### 🚀 Features

- Add README documentation and add back in other platform targets

## 0.2.6 (2024-09-12)

### 🚀 Features

- Updating changelog section config

## 0.2.5 (2024-09-12)

### Features

- Take out version number from artifact generation

## 0.2.4 (2024-09-12)

### Fixes

- small changes

## 0.2.3 (2024-09-12)

### Features

- version number parse was too late

## 0.2.2 (2024-09-12)

### Fixes

- test

## 0.2.1 (2024-09-12)

### Features

- test

## 0.2.0 (2024-09-12)

### Breaking Changes

- just ignore me for now

## 0.1.2 (2024-09-12)

### Features

- Fixing release steps

## 0.1.1 (2024-09-12)

### Features

- Initial experimental build
