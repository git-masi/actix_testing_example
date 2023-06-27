# Actix Web Automated Testing Examples

## About

This repo gives an example of how to write a unit test and an integration test for an Actix Web handler function.

The main goal is to use dependency injection to enable unit testing by implementing a common interface or "trait" as they are called in Rust.

Note that this may not be a desirable pattern to follow. I am not a Rust expert I'm just trying to understand what is possible.

Also note that the `async-trait` crate might not be required long term once the language has [standardized and stabilized async traits](https://blog.rust-lang.org/inside-rust/2023/05/03/stabilizing-async-fn-in-trait.html).

## Run the app/tests

Run the application (e.g. for manual testing purposes):

```
cargo watch -w src -x test
```

Eun all the tests in watch mode and get detailed errors for debugging:

```
RUST_LOG=debug cargo watch -w src -x test
```
