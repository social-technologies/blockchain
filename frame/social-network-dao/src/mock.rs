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

//! Test utilities

use super::*;
use crate as pallet_social_network_dao;
use std::cell::RefCell;

use frame_support::{
	parameter_types, ord_parameter_types,
	traits::{OnInitialize, OnFinalize, TestRandomness},
};
use sp_core::H256;
use sp_runtime::{
	Permill,
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use frame_system::EnsureSignedBy;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = u128;
type Balance = u64;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Assets: pallet_assets::{Module, Call, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		SocialGuardians: pallet_social_guardians::{Module, Call, Storage, Event<T>},
		Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
		SocialNetworkDao: pallet_social_network_dao::{Module, Call, Storage, Config<T>, Event<T>},
		Staking: pallet_staking::{Module, Call, Storage, Config<T>, Event<T>},
	}
);

parameter_types! {
	pub const CandidateDeposit: u64 = 25;
	pub const WrongSideDeduction: u64 = 2;
	pub const MaxStrikes: u32 = 2;
	pub const RotationPeriod: u64 = 4;
	pub const PeriodSpend: u64 = 1000;
	pub const MaxLockDuration: u64 = 100;
	pub const ChallengePeriod: u64 = 8;
	pub const BlockHashCount: u64 = 250;
	pub const ExistentialDeposit: u64 = 1;
	pub const SocialNetworkDaoModuleId: ModuleId = ModuleId(*b"st/sndao");
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

ord_parameter_types! {
	pub const FounderSetAccount: u128 = 1;
	pub const SuspensionJudgementSetAccount: u128 = 2;
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = Call;
	type Hashing = BlakeTwo256;
	type AccountId = u128;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type AccountData = pallet_balances::AccountData<u64>;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
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

thread_local! {
	static TEN_TO_FOURTEEN: RefCell<Vec<u128>> = RefCell::new(vec![10,11,12,13,14]);
}
parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: u64 = 1;
	pub const SpendPeriod: u64 = 2;
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TreasuryModuleId: ModuleId = ModuleId(*b"py/trsry");
	pub const BountyUpdatePeriod: u32 = 20;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: u64 = 1;
}

impl Config for Test {
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

pub struct EnvBuilder {
	members: Vec<u128>,
	balance: u64,
	balances: Vec<(u128, u64)>,
	pot: u64,
	max_members: u32,
}

impl EnvBuilder {
	pub fn new() -> Self {
		Self {
			members: vec![10],
			balance: 10_000,
			balances: vec![
				(10, 50),
				(20, 50),
				(30, 50),
				(40, 50),
				(50, 50),
				(60, 50),
				(70, 50),
				(80, 50),
				(90, 50),
			],
			pot: 0,
			max_members: 100,
		}
	}

	pub fn execute<R, F: FnOnce() -> R>(mut self, f: F) -> R {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		self.balances.push((SocialNetworkDao::account_id(), self.balance.max(self.pot)));
		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}.assimilate_storage(&mut t).unwrap();
		pallet_social_network_dao::GenesisConfig::<Test>{
			members: self.members,
			pot: self.pot,
			max_members: self.max_members,
		}.assimilate_storage(&mut t).unwrap();
		let mut ext: sp_io::TestExternalities = t.into();
		ext.execute_with(f)
	}
	#[allow(dead_code)]
	pub fn with_members(mut self, m: Vec<u128>) -> Self {
		self.members = m;
		self
	}
	#[allow(dead_code)]
	pub fn with_balances(mut self, b: Vec<(u128, u64)>) -> Self {
		self.balances = b;
		self
	}
	#[allow(dead_code)]
	pub fn with_pot(mut self, p: u64) -> Self {
		self.pot = p;
		self
	}
	#[allow(dead_code)]
	pub fn with_balance(mut self, b: u64) -> Self {
		self.balance = b;
		self
	}
	#[allow(dead_code)]
	pub fn with_max_members(mut self, n: u32) -> Self {
		self.max_members = n;
		self
	}
}

pub struct EnvBuilder2 {
	members: Vec<u128>,
	balance: u64,
	balances: Vec<(u128, u64)>,
	pot: u64,
	max_members: u32,
}

impl EnvBuilder2 {
	pub fn new() -> Self {
		Self {
			members: vec![10],
			balance: 10_000,
			balances: vec![(0, 100), (1, 98), (2, 1)],
			pot: 0,
			max_members: 100,
		}
	}

	pub fn execute<R, F: FnOnce() -> R>(mut self, f: F) -> R {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		self.balances.push((SocialNetworkDao::account_id(), self.balance.max(self.pot)));
		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}.assimilate_storage(&mut t).unwrap();
		pallet_social_network_dao::GenesisConfig::<Test>{
			members: self.members,
			pot: self.pot,
			max_members: self.max_members,
		}.assimilate_storage(&mut t).unwrap();
		let mut ext: sp_io::TestExternalities = t.into();
		ext.execute_with(f)
	}
	#[allow(dead_code)]
	pub fn with_members(mut self, m: Vec<u128>) -> Self {
		self.members = m;
		self
	}
	#[allow(dead_code)]
	pub fn with_balances(mut self, b: Vec<(u128, u64)>) -> Self {
		self.balances = b;
		self
	}
	#[allow(dead_code)]
	pub fn with_pot(mut self, p: u64) -> Self {
		self.pot = p;
		self
	}
	#[allow(dead_code)]
	pub fn with_balance(mut self, b: u64) -> Self {
		self.balance = b;
		self
	}
	#[allow(dead_code)]
	pub fn with_max_members(mut self, n: u32) -> Self {
		self.max_members = n;
		self
	}
}


/// Run until a particular block.
pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		SocialNetworkDao::on_initialize(System::block_number());
	}
}

/// Creates a bid struct using input parameters.
pub fn create_bid<AccountId, Balance>(
	value: Balance,
	who: AccountId,
	kind: BidKind<AccountId, Balance>
) -> Bid<AccountId, Balance>
{
	Bid {
		who,
		kind,
		value
	}
}
