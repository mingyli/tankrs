# tankrs

Wii Tank but Multiplayer and in Rust.

## Dependency Setup

tankrs uses `yarn` to manage dependencies. To get started:

1. Install yarn: `brew install yarn`
2. Install all dependencies: `yarn install`

tankrs is mainly written in `rust`. You will need the following

`rustfmt` for code formatting.
`rust-clippy` for writing idiomatic rust.

tankrs uses google/flatbuffers to serialize data to transfer over wire.
You will need to clone the GitHub repo and build the `flatc` binary.
[This commit](https://github.com/google/flatbuffers/commit/c3faa83463ca2556d6e7ab5b696cc311951f5857)
is confirmed to work, but HEAD should work unless otherwise noted.

## Formatting

tankrs uses [`prettier`](https://prettier.io/) for formatting.

To run a global format: `yarn run format`.

To check formatting: `yarn run check-format`.
