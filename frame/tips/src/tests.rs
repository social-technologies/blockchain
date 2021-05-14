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

//! Treasury pallet tests.

#![cfg(test)]

use crate as tips;
use super::*;
use std::cell::RefCell;
use frame_support::{
		assert_noop, assert_ok, ord_parameter_types, parameter_types, weights::Weight,
		traits::{Contains, TestRandomness},
};
use sp_runtime::Permill;
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
		TipsModTestInst: tips::{Module, Call, Storage, Event<T>},
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
const NET: u64 = 1000;
const HOURS: u64 = 3600;
const DAYS: u64 = 86400;
parameter_types! {
	pub const CandidateDeposit: Balance = 10 * NET;
	pub const WrongSideDeduction: Balance = 2 * NET;
	pub const MaxStrikes: u32 = 10;
	pub const RotationPeriod: BlockNumber = 80 * HOURS;
	pub const PeriodSpend: Balance = 500 * NET;
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
	pub const TipCountdown: u64 = 1;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: u64 = 1;
}
impl Config for Test {
	type MaximumReasonLength = MaximumReasonLength;
	type Tippers = TenToFourteen;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type DataDepositPerByte = DataDepositPerByte;
	type Event = Event;
	type WeightInfo = ();
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


fn last_event() -> RawEvent<u64, u128, H256> {
	System::events().into_iter().map(|r| r.event)
		.filter_map(|e| {
			if let Event::tips(inner) = e { Some(inner) } else { None }
		})
		.last()
		.unwrap()
}

#[test]
fn genesis_config_works() {
	EnvBuilder::new().execute(|| {
		assert_eq!(SocialNetworkDao::pot(), 0);
		assert_eq!(SocialNetworkDao::proposal_count(), 0);
	});
}

fn tip_hash() -> H256 {
	BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 3u128))
}

#[test]
fn tip_new_cannot_be_used_twice() {
	EnvBuilder::new().execute(|| {
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_ok!(TipsModTestInst::tip_new(Origin::signed(10), b"awesome.dot".to_vec(), 3, 10));
		assert_noop!(
			TipsModTestInst::tip_new(Origin::signed(11), b"awesome.dot".to_vec(), 3, 10),
			Error::<Test>::AlreadyKnown
		);
	});
}

#[test]
fn report_awesome_and_tip_works() {
	EnvBuilder::new().execute(|| {
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_ok!(TipsModTestInst::report_awesome(Origin::signed(0), b"awesome.dot".to_vec(), 3));
		assert_eq!(Balances::reserved_balance(0), 12);
		assert_eq!(Balances::free_balance(0), 88);

		// other reports don't count.
		assert_noop!(
			TipsModTestInst::report_awesome(Origin::signed(1), b"awesome.dot".to_vec(), 3),
			Error::<Test>::AlreadyKnown
		);

		let h = tip_hash();
		assert_ok!(TipsModTestInst::tip(Origin::signed(10), h.clone(), 10));
		assert_ok!(TipsModTestInst::tip(Origin::signed(11), h.clone(), 10));
		assert_ok!(TipsModTestInst::tip(Origin::signed(12), h.clone(), 10));
		assert_noop!(TipsModTestInst::tip(Origin::signed(9), h.clone(), 10), BadOrigin);
		System::set_block_number(2);
		assert_ok!(TipsModTestInst::close_tip(Origin::signed(100), h.into()));
		assert_eq!(Balances::reserved_balance(0), 0);
		assert_eq!(Balances::free_balance(0), 102);
		assert_eq!(Balances::free_balance(3), 8);
	});
}

#[test]
fn report_awesome_from_beneficiary_and_tip_works() {
	EnvBuilder::new().execute(|| {
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_ok!(TipsModTestInst::report_awesome(Origin::signed(0), b"awesome.dot".to_vec(), 0));
		assert_eq!(Balances::reserved_balance(0), 12);
		assert_eq!(Balances::free_balance(0), 88);
		let h = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
		assert_ok!(TipsModTestInst::tip(Origin::signed(10), h.clone(), 10));
		assert_ok!(TipsModTestInst::tip(Origin::signed(11), h.clone(), 10));
		assert_ok!(TipsModTestInst::tip(Origin::signed(12), h.clone(), 10));
		System::set_block_number(2);
		assert_ok!(TipsModTestInst::close_tip(Origin::signed(100), h.into()));
		assert_eq!(Balances::reserved_balance(0), 0);
		assert_eq!(Balances::free_balance(0), 110);
	});
}

#[test]
fn close_tip_works() {
	EnvBuilder::new().execute(|| {
		System::set_block_number(1);

		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_eq!(SocialNetworkDao::treasury_pot(), 100);

		assert_ok!(TipsModTestInst::tip_new(Origin::signed(10), b"awesome.dot".to_vec(), 3, 10));

		let h = tip_hash();

		assert_eq!(last_event(), RawEvent::NewTip(h));

		assert_ok!(TipsModTestInst::tip(Origin::signed(11), h.clone(), 10));

		assert_noop!(TipsModTestInst::close_tip(Origin::signed(0), h.into()), Error::<Test>::StillOpen);

		assert_ok!(TipsModTestInst::tip(Origin::signed(12), h.clone(), 10));

		assert_eq!(last_event(), RawEvent::TipClosing(h));

		assert_noop!(TipsModTestInst::close_tip(Origin::signed(0), h.into()), Error::<Test>::Premature);

		System::set_block_number(2);
		assert_noop!(TipsModTestInst::close_tip(Origin::none(), h.into()), BadOrigin);
		assert_ok!(TipsModTestInst::close_tip(Origin::signed(0), h.into()));
		assert_eq!(Balances::free_balance(3), 10);

		assert_eq!(last_event(), RawEvent::TipClosed(h, 3, 10));

		assert_noop!(TipsModTestInst::close_tip(Origin::signed(100), h.into()), Error::<Test>::UnknownTip);
	});
}

#[test]
fn slash_tip_works() {
	EnvBuilder::new().execute(|| {
		System::set_block_number(1);
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_eq!(SocialNetworkDao::treasury_pot(), 100);

		assert_eq!(Balances::reserved_balance(0), 0);
		assert_eq!(Balances::free_balance(0), 100);

		assert_ok!(TipsModTestInst::report_awesome(Origin::signed(0), b"awesome.dot".to_vec(), 3));

		assert_eq!(Balances::reserved_balance(0), 12);
		assert_eq!(Balances::free_balance(0), 88);

		let h = tip_hash();
		assert_eq!(last_event(), RawEvent::NewTip(h));

		// can't remove from any origin
		assert_noop!(
			TipsModTestInst::slash_tip(Origin::signed(0), h.clone()),
			BadOrigin,
		);

		// can remove from root.
		assert_ok!(TipsModTestInst::slash_tip(Origin::root(), h.clone()));
		assert_eq!(last_event(), RawEvent::TipSlashed(h, 0, 12));

		// tipper slashed
		assert_eq!(Balances::reserved_balance(0), 0);
		assert_eq!(Balances::free_balance(0), 88);
	});
}

#[test]
fn retract_tip_works() {
	EnvBuilder::new().execute(|| {
		// with report awesome
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_ok!(TipsModTestInst::report_awesome(Origin::signed(0), b"awesome.dot".to_vec(), 3));
		let h = tip_hash();
		assert_ok!(TipsModTestInst::tip(Origin::signed(10), h.clone(), 10));
		assert_ok!(TipsModTestInst::tip(Origin::signed(11), h.clone(), 10));
		assert_ok!(TipsModTestInst::tip(Origin::signed(12), h.clone(), 10));
		assert_noop!(TipsModTestInst::retract_tip(Origin::signed(10), h.clone()), Error::<Test>::NotFinder);
		assert_ok!(TipsModTestInst::retract_tip(Origin::signed(0), h.clone()));
		System::set_block_number(2);
		assert_noop!(TipsModTestInst::close_tip(Origin::signed(0), h.into()), Error::<Test>::UnknownTip);

		// with tip new
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_ok!(TipsModTestInst::tip_new(Origin::signed(10), b"awesome.dot".to_vec(), 3, 10));
		let h = tip_hash();
		assert_ok!(TipsModTestInst::tip(Origin::signed(11), h.clone(), 10));
		assert_ok!(TipsModTestInst::tip(Origin::signed(12), h.clone(), 10));
		assert_noop!(TipsModTestInst::retract_tip(Origin::signed(0), h.clone()), Error::<Test>::NotFinder);
		assert_ok!(TipsModTestInst::retract_tip(Origin::signed(10), h.clone()));
		System::set_block_number(2);
		assert_noop!(TipsModTestInst::close_tip(Origin::signed(10), h.into()), Error::<Test>::UnknownTip);
	});
}

#[test]
fn tip_median_calculation_works() {
	EnvBuilder::new().execute(|| {
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_ok!(TipsModTestInst::tip_new(Origin::signed(10), b"awesome.dot".to_vec(), 3, 0));
		let h = tip_hash();
		assert_ok!(TipsModTestInst::tip(Origin::signed(11), h.clone(), 10));
		assert_ok!(TipsModTestInst::tip(Origin::signed(12), h.clone(), 1000000));
		System::set_block_number(2);
		assert_ok!(TipsModTestInst::close_tip(Origin::signed(0), h.into()));
		assert_eq!(Balances::free_balance(3), 10);
	});
}

#[test]
fn tip_changing_works() {
	EnvBuilder::new().execute(|| {
		Balances::make_free_balance_be(&SocialNetworkDao::account_id(), 101);
		assert_ok!(TipsModTestInst::tip_new(Origin::signed(10), b"awesome.dot".to_vec(), 3, 10000));
		let h = tip_hash();
		assert_ok!(TipsModTestInst::tip(Origin::signed(11), h.clone(), 10000));
		assert_ok!(TipsModTestInst::tip(Origin::signed(12), h.clone(), 10000));
		assert_ok!(TipsModTestInst::tip(Origin::signed(13), h.clone(), 0));
		assert_ok!(TipsModTestInst::tip(Origin::signed(14), h.clone(), 0));
		assert_ok!(TipsModTestInst::tip(Origin::signed(12), h.clone(), 1000));
		assert_ok!(TipsModTestInst::tip(Origin::signed(11), h.clone(), 100));
		assert_ok!(TipsModTestInst::tip(Origin::signed(10), h.clone(), 10));
		System::set_block_number(2);
		assert_ok!(TipsModTestInst::close_tip(Origin::signed(0), h.into()));
		assert_eq!(Balances::free_balance(3), 10);
	});
}

#[test]
fn test_last_reward_migration() {
	use sp_storage::Storage;

	let mut s = Storage::default();

	#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
	pub struct OldOpenTip<
		AccountId: Parameter,
		Balance: Parameter,
		BlockNumber: Parameter,
		Hash: Parameter,
	> {
		/// The hash of the reason for the tip. The reason should be a human-readable UTF-8 encoded string. A URL would be
		/// sensible.
		reason: Hash,
		/// The account to be tipped.
		who: AccountId,
		/// The account who began this tip and the amount held on deposit.
		finder: Option<(AccountId, Balance)>,
		/// The block number at which this tip will close if `Some`. If `None`, then no closing is
		/// scheduled.
		closes: Option<BlockNumber>,
		/// The members who have voted for this tip. Sorted by AccountId.
		tips: Vec<(AccountId, Balance)>,
	}

	let reason1 = BlakeTwo256::hash(b"reason1");
	let hash1 = BlakeTwo256::hash_of(&(reason1, 10u64));

	let old_tip_finder = OldOpenTip::<u128, u64, u64, H256> {
		reason: reason1,
		who: 10,
		finder: Some((20, 30)),
		closes: Some(13),
		tips: vec![(40, 50), (60, 70)]
	};

	let reason2 = BlakeTwo256::hash(b"reason2");
	let hash2 = BlakeTwo256::hash_of(&(reason2, 20u64));

	let old_tip_no_finder = OldOpenTip::<u128, u64, u64, H256> {
		reason: reason2,
		who: 20,
		finder: None,
		closes: Some(13),
		tips: vec![(40, 50), (60, 70)]
	};

	let data = vec![
		(
			Tips::<Test>::hashed_key_for(hash1),
			old_tip_finder.encode().to_vec()
		),
		(
			Tips::<Test>::hashed_key_for(hash2),
			old_tip_no_finder.encode().to_vec()
		),
	];

	s.top = data.into_iter().collect();

	sp_io::TestExternalities::new(s).execute_with(|| {

		TipsModTestInst::migrate_retract_tip_for_tip_new();

		// Test w/ finder
		assert_eq!(
			Tips::<Test>::get(hash1),
			Some(OpenTip {
				reason: reason1,
				who: 10,
				finder: 20,
				deposit: 30,
				closes: Some(13),
				tips: vec![(40, 50), (60, 70)],
				finders_fee: true,
			})
		);

		// Test w/o finder
		assert_eq!(
			Tips::<Test>::get(hash2),
			Some(OpenTip {
				reason: reason2,
				who: 20,
				finder: Default::default(),
				deposit: 0,
				closes: Some(13),
				tips: vec![(40, 50), (60, 70)],
				finders_fee: false,
			})
		);
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