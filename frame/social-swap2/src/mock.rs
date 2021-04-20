use crate as pallet_social_swap2;
use sp_core::H256;
use frame_support::parameter_types;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system as system;
use sp_runtime::traits::Convert;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		Assets: pallet_assets::{Module, Call, Storage, Event<T>},
		SocialSwap: pallet_social_swap2::{Module, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
pub type Balance = u64;

impl system::Config for Test {
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
	type AccountId = u64;
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
		pub const MinimumPeriod: u64 = 5;
	}
impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
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
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type AssetDepositBase = AssetDepositBase;
	type AssetDepositPerZombie = AssetDepositPerZombie;
	type StringLimit = StringLimit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type WeightInfo = ();
}

parameter_types! {
    pub const MinimumLiquidity: u64 = 1000;
}

impl pallet_social_swap2::Config for Test {
	type Currency = Balances;
	type Event = Event;
	type FungibleToken = Assets;
	type MinimumLiquidity = MinimumLiquidity;
}
pub const ASSET_ID:u32 = 2;
pub const OWNER:u64 = 1;
pub const MAX_ZOMBIES:u32 = 3;
pub const MIN_BALANCE:u64 = 1;
pub const INITIAL_BALANCE:u64 = 100_000_0;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, INITIAL_BALANCE), (2, INITIAL_BALANCE)],
	}.assimilate_storage(&mut t).unwrap();

	pallet_assets::GenesisConfig::<Test> {
		assets: vec![(ASSET_ID, OWNER, OWNER, MAX_ZOMBIES, MIN_BALANCE)],
	}.assimilate_storage(&mut t).unwrap();

	t.into()
}
