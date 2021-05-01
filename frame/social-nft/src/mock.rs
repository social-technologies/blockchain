#![cfg(test)]

use frame_support::{ord_parameter_types, parameter_types};
use frame_system::{self as system};
use sp_core::hashing::blake2_128;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, Block as BlockT, IdentityLookup},
    BuildStorage,
};

use crate::{self as erc721, Config};
use chainbridge as bridge;
pub use pallet_balances as balances;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: system::{Module, Call, Event<T>},
        Balances: balances::{Module, Call, Storage, Config<T>, Event<T>},
		Assets: pallet_assets::{Module, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Erc721: erc721::{Module, Call, Storage, Event<T>},
    }
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
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
	type AccountId = u64; // u64 is not enough to hold bytes used to generate bounty account
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

ord_parameter_types! {
    pub const One: u64 = 1;
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

type Moment = u128;

parameter_types! {
		pub const MinimumPeriod: Moment = 5;
	}
impl pallet_timestamp::Config for Test {
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
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
	type Balance = u128;
	type AssetId = u32;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type AssetDepositBase = AssetDepositBase;
	type AssetDepositPerZombie = AssetDepositPerZombie;
	type StringLimit = StringLimit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type WeightInfo = ();
}

parameter_types! {
    pub Erc721Id: bridge::ResourceId = bridge::derive_resource_id(1, &blake2_128(b"NFT"));
}

impl Config for Test {
    type Event = Event;
    type Identifier = Erc721Id;
}

pub const USER_A: u64 = 0x1;
pub const USER_B: u64 = 0x2;
pub const USER_C: u64 = 0x3;
pub const ENDOWED_BALANCE: u64 = 100_000_000;
pub const ROYALTY:u128 = 2;
pub const ASSET_ID:u32 = 1;
pub const ACCOUNT1:u64 = 1;
pub const ACCOUNT2:u64 = 2;
pub const ACCOUNT3:u64 = 3;
pub const MAX_ZOMBIES:u32 = 3;
pub const MIN_BALANCE:u128 = 1;
pub const TOKEN0:u64 = 10;
pub const TOKEN1:u64 = 11;
pub const FEE_TO:u64 = 12;
pub const ADDRESS0:u64 = 13;
pub const TREASURY:u64 = 14;
pub const INITIAL_SUPPLY: u128  = 1_000_000_000_000_000_000_0000;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(USER_A, ENDOWED_BALANCE)/*, (ACCOUNT1, ENDOWED_BALANCE), (ACCOUNT2, ENDOWED_BALANCE)*/],
	}.assimilate_storage(&mut t).unwrap();

/*	pallet_assets::GenesisConfig::<Test> {
		assets: vec![(ASSET_ID, ACCOUNT1, ACCOUNT1, MAX_ZOMBIES, MIN_BALANCE)],
		accounts: vec![(ASSET_ID, ACCOUNT1, INITIAL_SUPPLY), (ASSET_ID, TREASURY, 0u128)],
	}.assimilate_storage(&mut t).unwrap();*/

	t.into()
}
