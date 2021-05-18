// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! bounties pallet tests.

#![cfg(test)]

use crate as pallet_bounties;
use super::*;
use std::cell::RefCell;

use frame_support::{
	assert_noop, assert_ok, ord_parameter_types, parameter_types, weights::Weight,
	traits::{Contains, ContainsLengthBound, OnInitialize, TestRandomness},
};

use sp_core::H256;
use sp_runtime::{
	Perbill, ModuleId,
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, BadOrigin},
};
use frame_system::EnsureSignedBy;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = u128;
type Balance = u64;
type BlockNumber = u64;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Assets: pallet_assets::{Module, Call, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		Bounties: pallet_bounties::{Module, Call, Storage, Event<T>},
		SocialGuardians: pallet_social_guardians::{Module, Call, Storage, Event<T>},
		Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
		SocialNetworkDao: pallet_social_network_dao::{Module, Call, Storage, Config<T>, Event<T>},
		Staking: pallet_staking::{Module, Call, Storage, Config<T>, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u128; // u64 is not enough to hold bytes used to generate bounty account
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}
parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const AssetDepositBase: u64 = 1;
	pub const AssetDepositPerZombie: u64 = 1;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: u64 = 1;
	pub const MetadataDepositPerByte: u64 = 1;
}

impl pallet_assets::Config for Test {
	type Currency = Balances;
	type Event = Event;
	type Balance = u64;
	type AssetId = u32;
	type ForceOrigin = frame_system::EnsureRoot<u128>;
	type AssetDepositBase = AssetDepositBase;
	type AssetDepositPerZombie = AssetDepositPerZombie;
	type StringLimit = StringLimit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type WeightInfo = ();
}

impl pallet_social_guardians::Config for Test {
	type Event = Event;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

impl pallet_session::historical::Config for Test {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Test>;
}

sp_runtime::impl_opaque_keys! {
	pub struct SessionKeys {
		pub foo: sp_runtime::testing::UintAuthorityId,
	}
}

pub struct TestSessionHandler;
impl pallet_session::SessionHandler<AccountId> for TestSessionHandler {
	const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[];

	fn on_genesis_session<Ks: sp_runtime::traits::OpaqueKeys>(_validators: &[(AccountId, Ks)]) {}

	fn on_new_session<Ks: sp_runtime::traits::OpaqueKeys>(
		_: bool,
		_: &[(AccountId, Ks)],
		_: &[(AccountId, Ks)],
	) {
	}

	fn on_disabled(_: usize) {}
}

impl pallet_session::Config for Test {
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Test, Staking>;
	type Keys = SessionKeys;
	type ShouldEndSession = pallet_session::PeriodicSessions<(), ()>;
	type NextSessionRotation = pallet_session::PeriodicSessions<(), ()>;
	type SessionHandler = TestSessionHandler;
	type Event = Event;
	type ValidatorId = AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Test>;
	type DisabledValidatorsThreshold = ();
	type WeightInfo = ();
}

pallet_staking_reward_curve::build! {
	const I_NPOS: sp_runtime::curve::PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000,
		max_inflation: 0_100_000,
		ideal_stake: 0_500_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}
parameter_types! {
	pub const RewardCurve: &'static sp_runtime::curve::PiecewiseLinear<'static> = &I_NPOS;
	pub const MaxNominatorRewardedPerValidator: u32 = 64;
	pub const UnsignedPriority: u64 = 1 << 20;
}

pub type Extrinsic = sp_runtime::testing::TestXt<Call, ()>;

impl<C> frame_system::offchain::SendTransactionTypes<C> for Test
where
	Call: From<C>,
{
	type OverarchingCall = Call;
	type Extrinsic = Extrinsic;
}

impl pallet_staking::Config for Test {
	type Currency = Balances;
	type UnixTime = pallet_timestamp::Module<Self>;
	type CurrencyToVote = frame_support::traits::SaturatingCurrencyToVote;
	type RewardRemainder = ();
	type Event = Event;
	type Slash = ();
	type Reward = ();
	type SessionsPerEra = ();
	type SlashDeferDuration = ();
	type SlashCancelOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type BondingDuration = ();
	type SessionInterface = Self;
	type RewardCurve = RewardCurve;
	type NextNewSession = Session;
	type ElectionLookahead = ();
	type Call = Call;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type UnsignedPriority = UnsignedPriority;
	type MaxIterations = ();
	type MinSolutionScoreBump = ();
	type OffchainSolutionWeightLimit = ();
	type WeightInfo = ();
}

pub struct TenToFourteen;
impl Contains<u128> for TenToFourteen {
	fn sorted_members() -> Vec<u128> {
		TEN_TO_FOURTEEN.with(|v| {
			v.borrow().clone()
		})
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn add(new: &u128) {
		TEN_TO_FOURTEEN.with(|v| {
			let mut members = v.borrow_mut();
			members.push(*new);
			members.sort();
		})
	}
}
impl ContainsLengthBound for TenToFourteen {
	fn max_len() -> usize {
		TEN_TO_FOURTEEN.with(|v| v.borrow().len())
	}
	fn min_len() -> usize { 0 }
}

thread_local! {
	static TEN_TO_FOURTEEN: RefCell<Vec<u128>> = RefCell::new(vec![10,11,12,13,14]);
}
const EARTH: u64 = 1000;
const HOURS: u64 = 3600;
const DAYS: u64 = 86400;
parameter_types! {
	pub const CandidateDeposit: Balance = 10 * EARTH;
	pub const WrongSideDeduction: Balance = 2 * EARTH;
	pub const MaxStrikes: u32 = 10;
	pub const RotationPeriod: BlockNumber = 80 * HOURS;
	pub const PeriodSpend: Balance = 500 * EARTH;
	pub const MaxLockDuration: BlockNumber = 36 * 30 * DAYS;
	pub const ChallengePeriod: BlockNumber = 7 * DAYS;
	pub const SocietyModuleId: ModuleId = ModuleId(*b"py/socie");
}
parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: u64 = 1;
	pub const SpendPeriod: u64 = 2;
	pub const Burn: Permill = Permill::from_percent(50);
	pub const SocialNetworkDaoModuleId: ModuleId = ModuleId(*b"st/sndao");
	pub const BountyUpdatePeriod: u32 = 20;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: u64 = 1;
	pub const MaximumReasonLength: u32 = 16384;
	pub const DataDepositPerByte: Balance = 1;
}
ord_parameter_types! {
	pub const FounderSetAccount: u128 = 1;
	pub const SuspensionJudgementSetAccount: u128 = 2;
}

impl pallet_social_network_dao::Config for Test {
	type Event = Event;
	type Currency = pallet_balances::Module<Self>;
	type Randomness = TestRandomness;
	type CandidateDeposit = CandidateDeposit;
	type WrongSideDeduction = WrongSideDeduction;
	type MaxStrikes = MaxStrikes;
	type PeriodSpend = PeriodSpend;
	type MembershipChanged = ();
	type RotationPeriod = RotationPeriod;
	type MaxLockDuration = MaxLockDuration;
	type FounderSetOrigin = EnsureSignedBy<FounderSetAccount, u128>;
	type SuspensionJudgementOrigin = EnsureSignedBy<SuspensionJudgementSetAccount, u128>;
	type ChallengePeriod = ChallengePeriod;
	type ModuleId = SocialNetworkDaoModuleId;

	type ApproveOrigin = frame_system::EnsureRoot<u128>;
	type RejectOrigin = frame_system::EnsureRoot<u128>;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();  // Just gets burned.
	type WeightInfo = ();
	type SpendFunds = ();
}

parameter_types! {
	pub const BountyDepositBase: u64 = 80;
	pub const BountyDepositPayoutDelay: u64 = 3;
}
impl Config for Test {
	type Event = Event;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type BountyCuratorDeposit = BountyCuratorDeposit;
	type BountyValueMinimum = BountyValueMinimum;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = ();
}

type SocialNetworkDaoError = pallet_social_network_dao::Error::<Test, pallet_social_network_dao::DefaultInstance>;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test>{
		// Total issuance will be 200 with treasury account initialized at ED.
		balances: vec![(0, 100), (1, 98), (2, 1)],
	}.assimilate_storage(&mut t).unwrap();
	pallet_social_network_dao::GenesisConfig::<Test>::default().assimilate_storage(&mut t).unwrap();
	t.into()
}

fn last_event() -> RawEvent<u64, u128> {
	System::events().into_iter().map(|r| r.event)
		.filter_map(|e| {
			if let Event::pallet_bounties(inner) = e { Some(inner) } else { None }
		})
		.last()
		.unwrap()
}

#[test]
fn genesis_config_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(SocialNetworkDao::treasury_pot(), 0);
		assert_eq!(SocialNetworkDao::proposal_count(), 0);
	});
}

#[test]
fn minting_works() {
	new_test_ext().execute_with(|| {
		// Check that accumulate works when we have Some value in Dummy already.
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_eq!(SocialNetworkDao::treasury_pot(), 100);
	});
}

#[test]
fn spend_proposal_takes_min_deposit() {
	new_test_ext().execute_with(|| {
		assert_ok!(SocialNetworkDao::propose_spend(Origin::signed(0), 1, 3));
		assert_eq!(Balances::free_balance(0), 99);
		assert_eq!(Balances::reserved_balance(0), 1);
	});
}

#[test]
fn spend_proposal_takes_proportional_deposit() {
	new_test_ext().execute_with(|| {
		assert_ok!(SocialNetworkDao::propose_spend(Origin::signed(0), 100, 3));
		assert_eq!(Balances::free_balance(0), 95);
		assert_eq!(Balances::reserved_balance(0), 5);
	});
}

#[test]
fn spend_proposal_fails_when_proposer_poor() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			SocialNetworkDao::propose_spend(Origin::signed(2), 100, 3),
			SocialNetworkDaoError::InsufficientProposersBalance,
		);
	});
}

#[test]
fn accepted_spend_proposal_ignored_outside_spend_period() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);

		assert_ok!(SocialNetworkDao::propose_spend(Origin::signed(0), 100, 3));
		assert_ok!(SocialNetworkDao::approve_proposal(Origin::root(), 0));

		<SocialNetworkDao as OnInitialize<u64>>::on_initialize(1);
		assert_eq!(Balances::free_balance(3), 0);
		assert_eq!(SocialNetworkDao::treasury_pot(), 100);
	});
}

#[test]
fn reject_already_rejected_spend_proposal_fails() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);

		assert_ok!(SocialNetworkDao::propose_spend(Origin::signed(0), 100, 3));
		assert_ok!(SocialNetworkDao::reject_proposal(Origin::root(), 0));
		assert_noop!(SocialNetworkDao::reject_proposal(Origin::root(), 0), SocialNetworkDaoError::InvalidIndex);
	});
}

#[test]
fn reject_non_existent_spend_proposal_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(SocialNetworkDao::reject_proposal(Origin::root(), 0),
		pallet_social_network_dao::Error::<Test, pallet_social_network_dao::DefaultInstance>::InvalidIndex);
	});
}

#[test]
fn accept_non_existent_spend_proposal_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(SocialNetworkDao::approve_proposal(Origin::root(), 0), SocialNetworkDaoError::InvalidIndex);
	});
}

#[test]
fn accept_already_rejected_spend_proposal_fails() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);

		assert_ok!(SocialNetworkDao::propose_spend(Origin::signed(0), 100, 3));
		assert_ok!(SocialNetworkDao::reject_proposal(Origin::root(), 0));
		assert_noop!(SocialNetworkDao::approve_proposal(Origin::root(), 0), SocialNetworkDaoError::InvalidIndex);
	});
}

#[test]
fn propose_bounty_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_eq!(SocialNetworkDao::treasury_pot(), 100);

		assert_ok!(Bounties::propose_bounty(Origin::signed(0), 10, b"1234567890".to_vec()));

		assert_eq!(last_event(), RawEvent::BountyProposed(0));

		let deposit: u64 = 85 + 5;
		assert_eq!(Balances::reserved_balance(0), deposit);
		assert_eq!(Balances::free_balance(0), 100 - deposit);

		assert_eq!(Bounties::bounties(0).unwrap(), Bounty {
			proposer: 0,
			fee: 0,
			curator_deposit: 0,
			value: 10,
			bond: deposit,
			status: BountyStatus::Proposed,
		});

		assert_eq!(Bounties::bounty_descriptions(0).unwrap(), b"1234567890".to_vec());

		assert_eq!(Bounties::bounty_count(), 1);
	});
}

#[test]
fn propose_bounty_validation_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_eq!(SocialNetworkDao::treasury_pot(), 100);

		assert_noop!(
			Bounties::propose_bounty(Origin::signed(1), 0, [0; 17_000].to_vec()),
			Error::<Test>::ReasonTooBig
		);

		assert_noop!(
			Bounties::propose_bounty(Origin::signed(1), 10, b"12345678901234567890".to_vec()),
			Error::<Test>::InsufficientProposersBalance
		);

		assert_noop!(
			Bounties::propose_bounty(Origin::signed(1), 0, b"12345678901234567890".to_vec()),
			Error::<Test>::InvalidValue
		);
	});
}

#[test]
fn close_bounty_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_noop!(Bounties::close_bounty(Origin::root(), 0), Error::<Test>::InvalidIndex);

		assert_ok!(Bounties::propose_bounty(Origin::signed(0), 10, b"12345".to_vec()));

		assert_ok!(Bounties::close_bounty(Origin::root(), 0));

		let deposit: u64 = 80 + 5;

		assert_eq!(last_event(), RawEvent::BountyRejected(0, deposit));

		assert_eq!(Balances::reserved_balance(0), 0);
		assert_eq!(Balances::free_balance(0), 100 - deposit);

		assert_eq!(Bounties::bounties(0), None);
		assert!(!pallet_social_network_dao::Proposals::<Test>::contains_key(0));

		assert_eq!(Bounties::bounty_descriptions(0), None);
	});
}

#[test]
fn assign_curator_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);

		assert_noop!(Bounties::propose_curator(Origin::root(), 0, 4, 4), Error::<Test>::InvalidIndex);

		assert_ok!(Bounties::propose_bounty(Origin::signed(0), 50, b"12345".to_vec()));

		assert_ok!(Bounties::approve_bounty(Origin::root(), 0));

		System::set_block_number(2);
		<SocialNetworkDao as OnInitialize<u64>>::on_initialize(2);

		assert_noop!(Bounties::propose_curator(Origin::root(), 0, 4, 50), Error::<Test>::InvalidFee);

		assert_ok!(Bounties::propose_curator(Origin::root(), 0, 4, 4));

		assert_eq!(Bounties::bounties(0).unwrap(), Bounty {
			proposer: 0,
			fee: 4,
			curator_deposit: 0,
			value: 50,
			bond: 85,
			status: BountyStatus::CuratorProposed {
				curator: 4,
			},
		});

		assert_noop!(Bounties::accept_curator(Origin::signed(1), 0), Error::<Test>::RequireCurator);
		assert_noop!(Bounties::accept_curator(Origin::signed(4), 0), pallet_balances::Error::<Test, _>::InsufficientBalance);

		Balances::make_free_balance_be(&4, 10);

		assert_ok!(Bounties::accept_curator(Origin::signed(4), 0));

		assert_eq!(Bounties::bounties(0).unwrap(), Bounty {
			proposer: 0,
			fee: 4,
			curator_deposit: 2,
			value: 50,
			bond: 85,
			status: BountyStatus::Active {
				curator: 4,
				update_due: 22,
			},
		});

		assert_eq!(Balances::free_balance(&4), 8);
		assert_eq!(Balances::reserved_balance(&4), 2);
	});
}

#[test]
fn unassign_curator_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_ok!(Bounties::propose_bounty(Origin::signed(0), 50, b"12345".to_vec()));

		assert_ok!(Bounties::approve_bounty(Origin::root(), 0));

		System::set_block_number(2);
		<SocialNetworkDao as OnInitialize<u64>>::on_initialize(2);

		assert_ok!(Bounties::propose_curator(Origin::root(), 0, 4, 4));

		assert_noop!(Bounties::unassign_curator(Origin::signed(1), 0), BadOrigin);

		assert_ok!(Bounties::unassign_curator(Origin::signed(4), 0));

		assert_eq!(Bounties::bounties(0).unwrap(), Bounty {
			proposer: 0,
			fee: 4,
			curator_deposit: 0,
			value: 50,
			bond: 85,
			status: BountyStatus::Funded,
		});

		assert_ok!(Bounties::propose_curator(Origin::root(), 0, 4, 4));

		Balances::make_free_balance_be(&4, 10);

		assert_ok!(Bounties::accept_curator(Origin::signed(4), 0));

		assert_ok!(Bounties::unassign_curator(Origin::root(), 0));

		assert_eq!(Bounties::bounties(0).unwrap(), Bounty {
			proposer: 0,
			fee: 4,
			curator_deposit: 0,
			value: 50,
			bond: 85,
			status: BountyStatus::Funded,
		});

		assert_eq!(Balances::free_balance(&4), 8);
		assert_eq!(Balances::reserved_balance(&4), 0); // slashed 2
	});
}

#[test]
fn expire_and_unassign() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_ok!(Bounties::propose_bounty(Origin::signed(0), 50, b"12345".to_vec()));

		assert_ok!(Bounties::approve_bounty(Origin::root(), 0));

		System::set_block_number(2);
		<SocialNetworkDao as OnInitialize<u64>>::on_initialize(2);

		assert_ok!(Bounties::propose_curator(Origin::root(), 0, 1, 10));
		assert_ok!(Bounties::accept_curator(Origin::signed(1), 0));

		assert_eq!(Balances::free_balance(1), 93);
		assert_eq!(Balances::reserved_balance(1), 5);

		System::set_block_number(22);
		<SocialNetworkDao as OnInitialize<u64>>::on_initialize(22);

		assert_noop!(Bounties::unassign_curator(Origin::signed(0), 0), Error::<Test>::Premature);

		System::set_block_number(23);
		<SocialNetworkDao as OnInitialize<u64>>::on_initialize(23);

		assert_ok!(Bounties::unassign_curator(Origin::signed(0), 0));

		assert_eq!(Bounties::bounties(0).unwrap(), Bounty {
			proposer: 0,
			fee: 10,
			curator_deposit: 0,
			value: 50,
			bond: 85,
			status: BountyStatus::Funded,
		});

		assert_eq!(Balances::free_balance(1), 93);
		assert_eq!(Balances::reserved_balance(1), 0); // slashed

	});
}

#[test]
fn extend_expiry() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		Balances::make_free_balance_be(&4, 10);
		assert_ok!(Bounties::propose_bounty(Origin::signed(0), 50, b"12345".to_vec()));

		assert_ok!(Bounties::approve_bounty(Origin::root(), 0));

		assert_noop!(Bounties::extend_bounty_expiry(Origin::signed(1), 0, Vec::new()), Error::<Test>::UnexpectedStatus);

		System::set_block_number(2);
		<SocialNetworkDao as OnInitialize<u64>>::on_initialize(2);

		assert_ok!(Bounties::propose_curator(Origin::root(), 0, 4, 10));
		assert_ok!(Bounties::accept_curator(Origin::signed(4), 0));

		assert_eq!(Balances::free_balance(4), 5);
		assert_eq!(Balances::reserved_balance(4), 5);

		System::set_block_number(10);
		<SocialNetworkDao as OnInitialize<u64>>::on_initialize(10);

		assert_noop!(Bounties::extend_bounty_expiry(Origin::signed(0), 0, Vec::new()), Error::<Test>::RequireCurator);
		assert_ok!(Bounties::extend_bounty_expiry(Origin::signed(4), 0, Vec::new()));

		assert_eq!(Bounties::bounties(0).unwrap(), Bounty {
			proposer: 0,
			fee: 10,
			curator_deposit: 5,
			value: 50,
			bond: 85,
			status: BountyStatus::Active { curator: 4, update_due: 30 },
		});

		assert_ok!(Bounties::extend_bounty_expiry(Origin::signed(4), 0, Vec::new()));

		assert_eq!(Bounties::bounties(0).unwrap(), Bounty {
			proposer: 0,
			fee: 10,
			curator_deposit: 5,
			value: 50,
			bond: 85,
			status: BountyStatus::Active { curator: 4, update_due: 30 }, // still the same
		});

		System::set_block_number(25);
		<SocialNetworkDao as OnInitialize<u64>>::on_initialize(25);

		assert_noop!(Bounties::unassign_curator(Origin::signed(0), 0), Error::<Test>::Premature);
		assert_ok!(Bounties::unassign_curator(Origin::signed(4), 0));

		assert_eq!(Balances::free_balance(4), 10); // not slashed
		assert_eq!(Balances::reserved_balance(4), 0);
	});
}

#[test]
fn genesis_funding_works() {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let initial_funding = 100;
	pallet_balances::GenesisConfig::<Test>{
		// Total issuance will be 200 with treasury account initialized with 100.
		balances: vec![(0, 100), (SocialNetworkDao::account_id(), initial_funding)],
	}.assimilate_storage(&mut t).unwrap();
	pallet_social_network_dao::GenesisConfig::<Test>::default().assimilate_storage(&mut t).unwrap();
	let mut t: sp_io::TestExternalities = t.into();

	t.execute_with(|| {
		assert_eq!(Balances::free_balance(SocialNetworkDao::account_id()), initial_funding);
		assert_eq!(SocialNetworkDao::treasury_pot(), initial_funding - Balances::minimum_balance());
	});
}
