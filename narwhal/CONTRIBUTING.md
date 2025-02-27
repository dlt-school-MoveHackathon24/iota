# Contributing to Narwhal / Bullshark

We want to make contributions as easy and transparent as possible.

## Pull Requests

We actively welcome your pull requests.

1. Fork the source repo and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. If you've changed APIs, update the documentation.
4. Ensure the test suite passes
5. Make sure your code lints

## Run the linter

The codebase uses the [Clippy](https://github.com/rust-lang/rust-clippy) linter
to ensure that common mistakes are caught. To install follow the instructions
on the Clippy repository. We make use of a cargo alias (defined in
.cargo/config) which sets some project-wide lints. To run the linter locally
you can use the following command:

```
cargo ci-clippy
```

## Run the formatter

To ensure that the codebase is following standard formatting properties, we use
[Rustfmt](https://github.com/rust-lang/rustfmt). To run the formatter locally
you can use the follow command:

```
cargo fmt --all
```

The above command will format and apply the changes directly. If you want to just
see the formatter recommendations, just run with the `--check` property:

```
cargo fmt --all -- --check
```

## Generating builders

The [derive_builder](https://crates.io/crates/derive_builder) crate has been used to
auto-generate builders (following the [builder design pattern](https://en.wikipedia.org/wiki/Builder_pattern)) for structs. Instead of having to write (lots) of boilerplate
code to create a builder, this is offered by the derive_builder and is the recommended
way to create builders for this repo. Examples can be found within the repo and on the
crate docs as well.

## Testing

One thing to note is that Narwhal uses snapshot testing for configs. See the beginning of the file
[config/tests/config_tests.rs](config/tests/config_tests.rs) for background and instructions if your PR breaks the test.

## Issues

We use GitHub issues to track public bugs. Please ensure your description is
clear and has sufficient instructions to be able to reproduce the issue.

## License

By contributing, you agree that your contributions will be licensed
under the LICENSE file in the root directory of this source tree.
