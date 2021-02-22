// This file is part of Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Substrate chain configurations.

use sc_chain_spec::ChainSpecExtension;
use sp_core::{Pair, Public, crypto::UncheckedInto, sr25519};
use serde::{Serialize, Deserialize};
use node_runtime::{
	AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, ContractsConfig, CouncilConfig,
	DemocracyConfig,GrandpaConfig, ImOnlineConfig, SessionConfig, SessionKeys, StakerStatus,
	StakingConfig, ElectionsConfig, IndicesConfig, SocietyConfig, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, wasm_binary_unwrap,
};
use node_runtime::Block;
use node_runtime::constants::currency::*;
use sc_service::ChainType;
use hex_literal::hex;
use sc_telemetry::TelemetryEndpoints;
use grandpa_primitives::{AuthorityId as GrandpaId};
use sp_consensus_babe::{AuthorityId as BabeId};
use pallet_im_online::sr25519::{AuthorityId as ImOnlineId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_runtime::{Perbill, traits::{Verify, IdentifyAccount}};

pub use node_primitives::{AccountId, Balance, Signature};
pub use node_runtime::GenesisConfig;

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<
	GenesisConfig,
	Extensions,
>;

/// ENERGY config generator
pub fn chi_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/CHI.json")[..])
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

fn staging_testnet_config_genesis() -> GenesisConfig {
	// stash, controller, session-key
	// generated with secret:
	// for i in 1 2 3 4 ; do for j in stash controller; do subkey inspect "$secret"/fir/$j/$i; done; done
	// and
	// for i in 1 2 3 4 ; do for j in session; do subkey --ed25519 inspect "$secret"//fir//$j//$i; done; done

	let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![(
		// 5D22ZznmkWbUjsqyRZQbwKhYmogKJXTaR9XQxE4jXDcJR5cM
		hex!["2a317d44ac572bfffb4c1c167796f319a1ec2f3910eb70549aa7365a42390c4c"].into(),
		// 5DbuzVy3Ng26JtePMBtntnSqDabUqdrGaWSKVwk7zmQYSoqN
		hex!["4409b0ed9c63a318a964505d4193ef105b4129a9ccc114c6092dd0263f08da1d"].into(),
		// 5FGniZwUfVT5wjRsc9KFCKRWpCgxLaYGB6cPCd5JoYnJU4tt
		hex!["8dec01afd8d64aa49326c58d3e53db3f414cc5610eec73b0e820c8a612628dbb"].unchecked_into(),
		// 5Fvjq3DCwjohh7vmgwd7eXJsEUJ9yrKSiiHisrq5E4TxEoCc
		hex!["aadd9bcc145b1a2e3ec809b6a1da6c478c03719b2991ff9a4afb73126ed11b75"].unchecked_into(),
		// 5FehTKkMJ1CyYbEF6U6LUHE51gjsEtJmApi43adorhTfa4wg
		hex!["9ea1b06eaacea92c1ec6ed994068ac4b7617f29781acd08660c427fa0fe72230"].unchecked_into(),
		// 5EZHEpPAqd4gDEF2AEfE22gU3gfw7bfhNJVV4NmrmHXXV6ju
		hex!["6e43b62ec13ddd79a8f4931c26dc656fd5c7ff80deed22e94c683cafb3196475"].unchecked_into(),
	),(
		// 5DLwZjUMrZSKGVnrTn7T9SWUrC4LGE35FCe5ZvVgGpari4UT
		hex!["389e4e2084c6a78ad577674a10bff738c4c5c0fda41f33e7c594b2fb28450a5a"].into(),
		// 5FmtnhuzUHBYma76D9uvHJrE3NZC4cmUTSYq6sdNU4eG7kDW
		hex!["a41e8fae660241b9a222972be0ceade99f4f8348e7540e7e45f7553d377b0a65"].into(),
		// 5DUwjPnoqYGExk9awrPt2bvFbPndN7BNkM925di53xAfakk8
		hex!["3eb8d44131635e437e887ea875a07fefb3a80d4d8ed125c5ca7320762d784ff9"].unchecked_into(),
		// 5GHL49TptY3BNtkUxmeUF27B4LLSw6u5MjvrutN3D3oLG1qi
		hex!["ba91bb4d0c7b53c94d3e0df497105754070284013664cf39f4dc9e50f1b4cc13"].unchecked_into(),
		// 5F52g1kNSvr5GyZHTRc9sBNJP9HcNBq681NxUYpKqVcjEv1E
		hex!["84f40aa9450147b22917f2edf508477f63531237809523bb87ed1e5ca5b40827"].unchecked_into(),
		// 5ECvriNLz96iDBm1LPDkUqxz7yxK4EUvL9xxc9tiXJNJurqi
		hex!["5ebe31d5fc33f3bd4d822d6ba11d42a928c48be3ee5baf4e858a2229586d5e0e"].unchecked_into(),
	),(
		// 5Fgrd9cHafGuwZZMjSwt4bvbCF59mV7mcko6oG4RMbdAMjXX
		hex!["a0470c04d391256fe6f85c96027a687726e10f0797b7dc85baaef681ed4e8444"].into(),
		// 5GTCwrui9jZshfGcp1q23rZQgJDNUe8dVYmgTQHdau2a6NQc
		hex!["c21a41bca17aa07adaffd5ad07cab5459ce337ec1ad7a7b18aa3a3cf07bd0d38"].into(),
		// 5GztshKKNv487yheDnLAfXX66UVTFiyKCveMz8DycbAn3E7h
		hex!["da454649dd6280a94d3d646f69ba7a962eafc2a15009a83df53797f8784984ca"].unchecked_into(),
		// 5CY4EvCcVYFyjjz5fZaa6Y24f4wr84msj6X8gGq2oom4XNBq
		hex!["14dc3e4663f3700d73e13ff676b18775b61dc45cac9f793db087f8d14acb7056"].unchecked_into(),
		// 5F9LJSXtXA9KxhEA1oYBLwVW6gZnCHHmHv3wZ25u8Z95Z1hh
		hex!["883c5d55ff230f5da6698a6022ca1a80e09f46f2145c03815a12d4b10c8a0974"].unchecked_into(),
		// 5CJxEzFWs5mE4XYZqtimQi9defNaJhz7EKCEEvz2AmC3tVnx
		hex!["0adddbf8484df340e890d004b2a0c80f64e66ae6eb51cbf5ed674a0137139f5d"].unchecked_into(),
	),(
		// 5DVFV48EXgTwM596Ly34LPeDVZf8GgNKX4ocG1PtSUWhJrDy
		hex!["3ef4971d135bf7a3a7ed15ab9896b9cb004dd986ad19d6f8ecac3fe828fbbe43"].into(),
		// 5DvWcMo5D4JQVcgQCsbPksKqWsSt2TkbNjaafryJBHEJiw7g
		hex!["5238a4d3ce4591ce5025cb655b813bb19b65038b064143a632662bb49311115a"].into(),
		// 5DNdTSfrKBPQUhEMBR1bTwgLbznUMcrJJTTcYCH2yD44aVEw
		hex!["39e7d6690159af463571c8d61a7224e954de0c7bb350f48111afe31f2152545c"].unchecked_into(),
		// 5CRjEeX22mTBqexu9ds1THkaVGzZknfpRuWA5kSjMndYxV2j
		hex!["1008cba1fea453ab044937b35b62018355e5852f5805d7634de78d05010c4241"].unchecked_into(),
		// 5GeijE4KFbr6QF9qd2GG6uPgjtnucdUHJwr22NwAzEPzGRud
		hex!["cae2363de6f972e4b02c59f7c4ec9affb5f2c13f20c451bc70c35530a7cbb62a"].unchecked_into(),
		// 5HQS6ber2UgkLCC1LCiHs1znCGq9EyKMxR7DoTVa1QVGVkM1
		hex!["ec3905e459a22a4d7caad6a9615e72edb776ead4723d1fcb958201fd64346264"].unchecked_into(),
	)];

	// generated with secret: subkey inspect "$secret"/fir
	let root_key: AccountId = hex![
		// 5C7R7a9k6ntba9StKrNvNBP2H3Y4wuLvdmFYKBH3XiUQbEdr
		"02115fc5e45c37015ccf1a78107565d0577d0c75c56fa36683e004d4a365321c"
	].into();

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(
		initial_authorities,
		root_key,
		Some(endowed_accounts),
		false,
	)
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
	let boot_nodes = vec![];
	ChainSpec::from_genesis(
		"Staging Testnet",
		"staging_testnet",
		ChainType::Live,
		staging_testnet_config_genesis,
		boot_nodes,
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
			.expect("Staging telemetry url is valid; qed")),
		None,
		None,
		Default::default(),
	)
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(seed: &str) -> (
	AccountId,
	AccountId,
	GrandpaId,
	BabeId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	enable_println: bool,
) -> GenesisConfig {
	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		]
	});
	let num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: Balance = 7_777_377 * ENERGY;
	const STASH: Balance = 100 * ENERGY;

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			balances: endowed_accounts.iter().cloned()
				.map(|k| (k, ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
		}),
		pallet_indices: Some(IndicesConfig {
			indices: vec![],
		}),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.0.clone(), session_keys(
					x.2.clone(),
					x.3.clone(),
					x.4.clone(),
					x.5.clone(),
				))
			}).collect::<Vec<_>>(),
		}),
		pallet_staking: Some(StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)
			}).collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			.. Default::default()
		}),
		pallet_democracy: Some(DemocracyConfig::default()),
		pallet_elections_phragmen: Some(ElectionsConfig {
			members: endowed_accounts.iter()
						.take((num_endowed_accounts + 1) / 2)
						.cloned()
						.map(|member| (member, STASH))
						.collect(),
		}),
		pallet_collective_Instance1: Some(CouncilConfig::default()),
		pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
			members: endowed_accounts.iter()
						.take((num_endowed_accounts + 1) / 2)
						.cloned()
						.collect(),
			phantom: Default::default(),
		}),
		pallet_contracts: Some(ContractsConfig {
			current_schedule: pallet_contracts::Schedule {
				enable_println, // this should only be enabled on development chains
				..Default::default()
			},
		}),
		pallet_sudo: Some(SudoConfig {
			key: root_key,
		}),
		pallet_babe: Some(BabeConfig {
			authorities: vec![],
		}),
		pallet_im_online: Some(ImOnlineConfig {
			keys: vec![],
		}),
		pallet_authority_discovery: Some(AuthorityDiscoveryConfig {
			keys: vec![],
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		pallet_membership_Instance1: Some(Default::default()),
		pallet_treasury: Some(Default::default()),
		pallet_society: Some(SocietyConfig {
			members: endowed_accounts.iter()
						.take((num_endowed_accounts + 1) / 2)
						.cloned()
						.collect(),
			pot: 0,
			max_members: 999,
		}),
		pallet_vesting: Some(Default::default()),
	}
}

fn development_config_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			authority_keys_from_seed("Alice"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		true,
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		development_config_genesis,
		vec![],
		None,
		None,
		None,
		Default::default(),
	)
}

fn local_testnet_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			authority_keys_from_seed("Alice"),
			authority_keys_from_seed("Bob"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		false,
	)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		local_testnet_genesis,
		vec![],
		None,
		None,
		None,
		Default::default(),
	)
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate::service::{new_full_base, new_light_base, NewFullBase};
	use sc_service_test;
	use sp_runtime::BuildStorage;

	fn local_testnet_genesis_instant_single() -> GenesisConfig {
		testnet_genesis(
			vec![
				authority_keys_from_seed("Alice"),
			],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
			false,
		)
	}

	/// Local testnet config (single validator - Alice)
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis_instant_single,
			vec![],
			None,
			None,
			None,
			Default::default(),
		)
	}

	/// Local testnet config (multivalidator Alice + Bob)
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis,
			vec![],
			None,
			None,
			None,
			Default::default(),
		)
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		sc_service_test::connectivity(
			integration_test_config_with_two_authorities(),
			|config| {
				let NewFullBase { task_manager, client, network, transaction_pool, .. }
					= new_full_base(config,|_, _| ())?;
				Ok(sc_service_test::TestNetComponents::new(task_manager, client, network, transaction_pool))
			},
			|config| {
				let (keep_alive, _, client, network, transaction_pool) = new_light_base(config)?;
				Ok(sc_service_test::TestNetComponents::new(keep_alive, client, network, transaction_pool))
			}
		);
	}

	#[test]
	fn test_create_development_chain_spec() {
		development_config().build_storage().unwrap();
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		local_testnet_config().build_storage().unwrap();
	}

	#[test]
	fn test_staging_test_net_chain_spec() {
		staging_testnet_config().build_storage().unwrap();
	}
}
