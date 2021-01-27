use crate::{Module, Trait};
use frame_support::{impl_outer_origin, ord_parameter_types, parameter_types, weights::Weight};
use frame_system as system;
use frame_system::{EnsureOneOf, EnsureRoot, EnsureSignedBy};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

impl_outer_origin! {
    pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MaxRegistrars: u32 = 2;
    pub const MinUsernameLength: u32 = 3;
    pub const MaxUsernameLength: u32 = 10;
}

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

ord_parameter_types! {
    pub const One: u64 = 1;
    pub const Two: u64 = 2;
}
type EnsureOneOrRoot = EnsureOneOf<u64, EnsureRoot<u64>, EnsureSignedBy<One, u64>>;
type EnsureTwoOrRoot = EnsureOneOf<u64, EnsureRoot<u64>, EnsureSignedBy<Two, u64>>;
impl Trait for Test {
    type Event = ();
    type MaxRegistrars = MaxRegistrars;
    type RegistrarOrigin = EnsureOneOrRoot;
    type ForceOrigin = EnsureTwoOrRoot;
    type MinUsernameLength = MinUsernameLength;
    type MaxUsernameLength = MaxUsernameLength;
    type WeightInfo = ();
}

pub type UsernameRegistry = Module<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
