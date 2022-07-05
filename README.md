# Serverless Bamboo Time Off Requests Alerter

Post Time Off Requests from Bamboo into a Slack channel

Uses the [Serverless Framework](https://www.serverless.com/) and is built in Rust

## Install Pre-requisites

### Packages

Please use install these necessary components if not already installed:

- zip
- clang (usually via llvm)
- rust (with rustup)

### Rust aarch64 musl target for cross-compliation of static binary to arm64:

```sh
rustup target add aarch64-unknown-linux-musl
```

## Configuration

Deployment-specific config items should be placed in a `config.STAGE.yaml`
file, where STAGE is the deployment stage used (defaults to `dev`).

Feel free to copy the `config.example.yaml` file and modify the values therein.

## Check

Make sure the program compiles on your machine and given any code changes you've made:

```sh
make
```

## Install

```sh
# set AWS_PROFILE here if necessary
AWS_PROFILE=... make deploy
```

## License

[ISC](LICENSE)
