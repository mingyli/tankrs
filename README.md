# tankrs

Wii Tank but Multiplayer and in Rust.

## Dependency Setup

tankrs uses `yarn` to manage dependencies. To get started:

1. Install yarn: `brew install yarn`
2. Install all dependencies: `yarn install`

tankrs is mainly written in `rust`. You will need the following

- `rustfmt` for code formatting.
- `rust-clippy` for writing idiomatic rust.

tankrs uses `protobuf` to serialize data to transfer over wire. You may need to compile
`protoc` in order to generate protobuffers.

## Client

Run `make all` in client/ to generate flatbuffers and app.js .
You can then open client/app/app.html.

## Server

Run `cargo run` in the root directory.
This will take care of all build requirements as well.

## Formatting

tankrs uses [`prettier`](https://prettier.io/) for formatting.

To run a global format: `yarn run format`.

To check formatting: `yarn run check-format`.
