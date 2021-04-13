use crate as pallet_social_champions;
use frame_support::{parameter_types, traits::GenesisBuild};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Assets: pallet_assets::{Module, Call, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        SocialChampions: pallet_social_champions::{Module, Call, Storage, Event<T>},
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

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = u64;
    type DustRemoval = ();
    type Event = Event;
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

impl pallet_social_champions::Config for Test {
    type Event = Event;
}

pub type ValidatorRegistry = pallet_social_champions::Module<Test>;

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        Self
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
        let (owner, admin, max_zombies, min_balance) = (1, 1, 0, 100);
        pallet_assets::GenesisConfig::<Test> {
            assets: (1..=17).map(|id| (id, owner, admin, max_zombies, min_balance)).collect(),
        }.assimilate_storage(&mut t).unwrap();
        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    ExtBuilder::default().build()
}
