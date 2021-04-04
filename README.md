# Social 🌎 Network v3 &middot; [![GitHub license](https://img.shields.io/badge/license-GPL3%2FApache2-blue)](LICENSE) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](docs/CONTRIBUTING.adoc)

Social 🌎 Network v3 is a next-generation governance, economic, and social system for humanity built on Polkadot Substrate. To learn more [read the whitepaper](https://bit.ly/2Jheagq) 🚀

![Substrate Builders Program](/docs/SBP_M2.png)

## Earn NET by running a Node

Social 🌎 Network v3 is powered by a decentralized ledger enabling anyone in the world to participate by running the node software in this repository. This removes the need for a central party or middleman to extract rent or censor the network for personal gain, control, or power. The blockchain is capable of securing and maintaining a single source of truth, globally, with 6 second finality using a byzantine fault tolerant, nominated Proof-of-Stake consensus algorithm.

Node operators are incentivized with NET tokens for keeping the software running, and earn greater rewards the more reliable their nodes are over time. In addition to staking and transaction fees, Node operators can choose to support Societies by pinning their data on IPFS and earn Social Tokens for doing so.

## Running a NET Node

To get started, you can follow the steps depending on your operating system:

### Macos

Open the Terminal application and execute the following commands:
```
# Install Homebrew if necessary https://brew.sh/
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"

# Make sure Homebrew is up-to-date, install openssl and cmake
brew update
brew install openssl cmake
```

### Ubuntu/Debian

Use a terminal shell to execute the following commands:

```
sudo apt update
# May prompt for location information
sudo apt install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl libz-dev
```

### Arch Linux

Run these commands from a terminal:

```
pacman -Syu --needed --noconfirm cmake gcc openssl-1.0 pkgconf git clang
export OPENSSL_LIB_DIR="/usr/lib/openssl-1.0"
export OPENSSL_INCLUDE_DIR="/usr/include/openssl-1.0"
```

### Windows

Please see [this guide](https://substrate.dev/docs/en/knowledgebase/getting-started/windows-users) to get setup using Windows.

## Setup your Rust Environment

Social 🌎 Network v3 software is built in rust, so node operators must setup their tooling as follows:

Install rust with the following:
```
# Install
curl https://sh.rustup.rs -sSf | sh
# Configure
source ~/.cargo/env
# Configure Rust toolchain to the one needed for The Social Blockchain
rustup install nightly-2021-01-25
rustup default nightly-2021-01-25
# Install WebAssembly (WASM)
rustup target add wasm32-unknown-unknown --toolchain nightly-2021-01-25
```

## Download Social 🌎 Network v3

To get a copy of Social 🌎 Network v3 software on your machine:

```
git clone https://github.com/social-network/v3.git
```

The runtime of the blockchain is compiled down to [Web Assembly (WASM)](https://webassembly.org/) so it can be ran in embedded devices, or even the browser. To compile it run:

```
cd blockchain
WASM_BUILD_TOOLCHAIN=nightly-2021-01-25 cargo build --release
```

Note if you are doing other blockchain development work, you will need to update your toolchain to match our version for the compilation to work. Feel free to join the #support circle in the [Social Technologies Society](https://social.network/join/tech) if you need help getting setup.

## Help Build Social 🌎 Network v3

As a decentralized project, we welcome all contributions that help us reach our mission faster, but ask that you follow our values. See our [welcome blog](https://blog.social.network/welcome-to-social) for more details.

To jump right in, visit our [jobs](https://social.network/jobs) page to find a job or bounty that you think you can complete. Contributors will earn NET tokens to participate in the platform governance and help steward the decentralized technical development.

## Security

The security policy and procedures can be found in [`docs/SECURITY.md`](docs/SECURITY.md).

## License

- Substrate Primitives (`sp-*`), Frame (`frame-*`) and the pallets (`pallets-*`), binaries (`/bin`) and all other utilities are licensed under [Apache 2.0](LICENSE-APACHE2).
- Substrate Client (`/client/*` / `sc-*`) is licensed under [GPL v3.0 with a classpath linking exception](LICENSE-GPL3).

The reason for the split-licensing is to ensure that for the vast majority of teams using Substrate to create feature-chains, then all changes can be made entirely in Apache2-licensed code, allowing teams full freedom over what and how they release and giving licensing clarity to commercial teams.

In the interests of the community, we require any deeper improvements made to Substrate's core logic (e.g. Substrate's internal consensus, crypto or database code) to be contributed back so everyone can benefit.

### Type definitions

In order to be compatible with polkadot.js the following type definitions will need to be added:

```
{
  "AttributeTransaction": {
    "signature": "Signature",
    "name": "Vec<u8>",
    "value": "Vec<u8>",
    "validity": "u32",
    "signer": "AccountId",
    "identity": "AccountId"
  },
  "Attribute": {
    "name": "Vec<u8>",
    "value": "Vec<u8>",
    "validity": "BlockNumber",
    "creation": "Moment",
    "nonce": "u64"
  },
  "SwapId": "u64",
  "Swap": {
    "token_id": "TokenId",
    "swap_token": "TokenId",
    "account": "AccountId"
  },
  "SocialId": "u32",
  "SocialTokenId": "u32",
  "SocialTokenBalance": "u128",
  "RegistrarIndex": "u32",
  "Judgement": {
    "_enum": [
      "Requested",
      "Approved"
    ]
  },
  "JudgementItem": "(RegistrarIndex, Judgement)",
  "Registration": {
    "judgements": "Vec<JudgementItem>",
    "account_id": "AccountId"
  },
  "Bloom": "H256",
  "Log": {
    "address": "H160",
    "topics": "Vec<H256>",
    "data": "Bytes"
  },
  "Receipt": {
    "state_root": "H256",
    "used_gas": "U256",
    "logs_bloom": "Bloom",
    "logs": "Vec<Log>"
  },
  "TransactionAction": {
    "_enum": {
	"Call": "H160",
	"Create": "Null"
    }
  },
  "TransactionRecoveryId": "u64",
  "TransactionSignature": {
    "v": "TransactionRecoveryId",
    "r": "H256",
    "s": "H256"
  },
  "Transaction": {
    "nonce": "U256",
    "gas_price": "U256",
    "gas_limit": "U256",
    "action": "TransactionAction",
    "value": "U256",
    "input": "Bytes",
    "signature": "TransactionSignature"
  },
  "TransactionStatus": {
    "transaction_hash": "H256",
    "transaction_index": "u32",
    "from": "H160",
    "to": "Option<H160>",
    "contract_address": "Option<H160>",
    "logs": "Vec<Log>",
    "logs_bloom": "Bloom"
  },
  "Id": "AuthorityId",
  "ChainId": "u8",
  "ResourceId": "Vec<u8>",
  "ExchangeId": "u64",
  "CurrencyOf": "Balance",
  "NftId": "U256",
  "Erc721Token": {
    "id": "NftId",
    "metadata": "Vec<u8>"
  }
}
```
