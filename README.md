# DepRank Backend Server

The backend server for DepRank, It is written in Rust, with the Axum framework.

[![License](https://img.shields.io/github/license/deprank/backend)](https://github.com/deprank/backend/blob/master/LICENSE)
[![GitHub
contributors](https://img.shields.io/github/contributors/deprank/backend)](https://github.com/deprank/backend/graphs/contributors)
[![GitHub
issues](https://img.shields.io/github/issues/deprank/backend)](https://github.com/deprank/backend/issues)

## Installation

Prebuilt binaries Windows, Linux and macOS can be downloaded from the
[Github release page](https://github.com/deprank/backend/releases/latest).
If there is no distro package available in your preferred manager,
you need [Rust and cargo](https://www.rust-lang.org/tools/install) to build it.

### Install from source:

1. Clone the repository with `git clone https://github.com/deprank/backend.git`
2. From the `backend` directory, run `cargo build --release` to
   build the application in release mode.
3. After a successful compilation, launch the executable with:
   `target/release/deprank-server`.

### Install with cargo

To get the latest bug fixes and features, you can install the development
version from git. However, this is not fully tested. That means you're probably
going to have more bugs despite having the latest bug fixes.

```
cargo install --git https://github.com/deprank/backend
```

This will download the source from the main branch, build and install it in
Cargo's global binary directory (`~/.cargo/bin/` by default).

## Usage

```text
Usage: deprank-server [OPTIONS]

Options:
      --port <PORT>
          The Server port [env: DRK_PORT] [default: 8080]
      --starknet-rpc-url <STARKNET_RPC_URL>
          URL of the Starknet JSON-RPC endpoint [env: STARKNET_RPC_URL]
      --starknet-private-key <STARKNET_PRIVATE_KEY>
          Private key of the Starknet account [env: STARKNET_PRIVATE_KEY]
      --starknet-account-address <STARKNET_ACCOUNT_ADDRESS>
          Address of the Starknet account [env: STARKNET_ACCOUNT_ADDRESS]
      --starknet-chain-id <STARKNET_CHAIN_ID>
          Chain ID of the Starknet network [env: STARKNET_CHAIN_ID]
      --allocation-contract-address <ALLOCATION_CONTRACT_ADDRESS>
          Address of the Allocation contract [env: ALLOCATION_CONTRACT_ADDRESS]
      --inquire-contract-address <INQUIRE_CONTRACT_ADDRESS>
          Address of the Inquire contract [env: INQUIRE_CONTRACT_ADDRESS]
      --receipt-contract-address <RECEIPT_CONTRACT_ADDRESS>
          Address of the Receipt contract [env: RECEIPT_CONTRACT_ADDRESS]
      --sign-contract-address <SIGN_CONTRACT_ADDRESS>
          Address of the Sign contract [env: SIGN_CONTRACT_ADDRESS]
      --workflow-contract-address <WORKFLOW_CONTRACT_ADDRESS>
          Address of the Workflow contract [env: WORKFLOW_CONTRACT_ADDRESS]
      --cache-dir <CACHE_DIR>
          Base directory for storing cached repositories [env: CACHE_DIR]
      --github-token <GITHUB_TOKEN>
          A personal token to use for authentication [env: GITHUB_TOKEN]
  -h, --help
          Print help
```

## Development

To build this project, you will need to install the following pre-requisites:
[Git](https://git-scm.com/downloads),
[Rust](https://www.rust-lang.org/tools/install) and
[Just](https://github.com/casey/just).

After cloning the repository, you can simply run `just` in the package directory
to list all available commands. For your first local build, please run `just
install` command to install the dependencies for this project.

## Contributing

If anything feels off, or if you feel that some functionality is missing, please
check out the [contributing page](CONTRIBUTING.md). There you will find
instructions for sharing your feedback, building the project locally, and
submitting pull requests to the project.

## License

Copyright (c) The DepRank Authors. All rights reserved.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
