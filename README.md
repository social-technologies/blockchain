# The Social Network &middot; [![GitHub license](https://img.shields.io/badge/license-GPL3%2FApache2-blue)](LICENSE) [![GitLab Status](https://gitlab.parity.io/parity/substrate/badges/master/pipeline.svg)](https://gitlab.parity.io/parity/substrate/pipelines) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](docs/CONTRIBUTING.adoc)

The Social Network is a next-generation governance, economic, and social system for humanity 🚀. [Read the Whitepaper](https://bit.ly/2Jheagq).

## Contributions & Code of Conduct

Please follow the contributions guidelines as outlined in [`docs/CONTRIBUTING.adoc`](docs/CONTRIBUTING.adoc). In all communications and contributions, this project follows the [Contributor Covenant Code of Conduct](docs/CODE_OF_CONDUCT.md).

## Security

The security policy and procedures can be found in [`docs/SECURITY.md`](docs/SECURITY.md).

## License

- Substrate Primitives (`sp-*`), Frame (`frame-*`) and the pallets (`pallets-*`), binaries (`/bin`) and all other utilities are licensed under [Apache 2.0](LICENSE-APACHE2).
- Substrate Client (`/client/*` / `sc-*`) is licensed under [GPL v3.0 with a classpath linking exception](LICENSE-GPL3).

The reason for the split-licensing is to ensure that for the vast majority of teams using Substrate to create feature-chains, then all changes can be made entirely in Apache2-licensed code, allowing teams full freedom over what and how they release and giving licensing clarity to commercial teams.

In the interests of the community, we require any deeper improvements made to Substrate's core logic (e.g. Substrate's internal consensus, crypto or database code) to be contributed back so everyone can benefit.

## Development

### Installation

Follow the [installation](https://substrate.dev/docs/en/knowledgebase/getting-started/) instructions.

### Run a local node

```bash
cargo run -- --dev --tmp
```

### Type definitions

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
  "TokenId": "u64",
  "SwapId": "u64",
  "TokenBalance": "u64",
  "Swap": {
    "token_id": "TokenId",
    "swap_token": "TokenId",
    "account": "AccountId"
  },
  "MissionId": "u32",
  "MissionTokenId": "u32",
  "MissionTokenBalance": "u128",
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
  "ResourceId": "Vec<u8>"
}
```
