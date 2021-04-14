use crate as pallet_social_swap;
use sp_core::H256;
use frame_support::parameter_types;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system as system;
use sp_runtime::ModuleId;
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
		SocialTokens: pallet_social_tokens::{Module, Call, Storage, Event<T>},
		SocialSwap: pallet_social_swap::{Module, Call, Storage, Event<T>},
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
	pub const MaxSocialTokensSupply: u128 = 7_777_777_777;
}

impl pallet_social_tokens::Config for Test {
	type Event = Event;
	type Balance = u64;
	type SocialTokenId = u32;
	type ExistentialDeposit = ExistentialDeposit;
	type OnNewAccount = ();
	type MaxSocialTokensSupply = MaxSocialTokensSupply;
}

parameter_types! {
	pub const ExchangeModuleId: ModuleId = ModuleId(*b"exchange");
}

pub struct BalanceHandler;
impl Convert<Balance, u64> for BalanceHandler {
	fn convert(a: Balance) -> u64 {
		a
	}
}

impl pallet_social_swap::Config for Test {
	type Currency = Balances;
	type ModuleId = ExchangeModuleId;
	type Event = Event;
	type FungibleToken = SocialTokens;
	type Handler = BalanceHandler;
	type ExchangeId = u64;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![],
	}.assimilate_storage(&mut t).unwrap();
	t.into()
}
