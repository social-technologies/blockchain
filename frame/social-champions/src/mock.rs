use crate as pallet_social_champions;
use frame_support::{ord_parameter_types, parameter_types};
use frame_system as system;
use frame_system::{EnsureOneOf, EnsureRoot, EnsureSignedBy};
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
		SocialTokens: pallet_social_tokens::{Module, Call, Storage, Event<T>},
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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

parameter_types! {
    pub const ExistentialDepositOfSocialTokens: u128 = 1_000;
    pub const MaxSocialTokensSupply: u128 = 7_777_777_777;
}

ord_parameter_types! {
    pub const One: u64 = 1;
}
type EnsureOneOrRoot = EnsureOneOf<u64, EnsureRoot<u64>, EnsureSignedBy<One, u64>>;

impl pallet_social_tokens::Config for Test {
    type Event = Event;
    type Balance = u128;
    type SocialTokenId = u32;
    type ExistentialDeposit = ExistentialDepositOfSocialTokens;
    type OnNewAccount = ();
    type MaxSocialTokensSupply = MaxSocialTokensSupply;
    type SocialCreatorOrigin = EnsureOneOrRoot;
}

impl pallet_social_champions::Config for Test {
    type Event = Event;
}

pub type ValidatorRegistry = pallet_social_champions::Module<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
