use crate::{Module, Trait};
use frame_support::{
    impl_outer_dispatch, impl_outer_origin, parameter_types,
    traits::{Contains, ContainsLengthBound},
    weights::Weight,
};
use pallet_session::{SessionHandler, SessionManager, ShouldEndSession};
use pallet_staking::EraIndex;
use sp_core::{crypto::key_types::DUMMY, H256};
use sp_runtime::{
    curve::PiecewiseLinear,
    impl_opaque_keys,
    testing::{Header, TestXt, UintAuthorityId},
    traits::{BlakeTwo256, Convert, IdentityLookup, OpaqueKeys},
    ModuleId, Perbill, Percent, Permill, RuntimeAppPublic, SaturatedConversion,
};
use sp_staking::SessionIndex;
use std::cell::RefCell;

use frame_system as system;

impl_outer_origin! {
    pub enum Origin for Test where system = frame_system {}
}

impl_outer_dispatch! {
    pub enum Call for Test where origin: Origin {
        staking::Staking,
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(16);
}

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = Call;
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
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

parameter_types! {
    pub const ExistentialDepositC: u64 = 1;
}
thread_local! {
    static TEN_TO_FOURTEEN: RefCell<Vec<u128>> = RefCell::new(vec![10,11,12,13,14]);

    pub static VALIDATORS: RefCell<Vec<u64>> = RefCell::new(vec![1, 2, 3]);
    pub static NEXT_VALIDATORS: RefCell<Vec<u64>> = RefCell::new(vec![1, 2, 3]);
    pub static AUTHORITIES: RefCell<Vec<UintAuthorityId>> =
        RefCell::new(vec![UintAuthorityId(1), UintAuthorityId(2), UintAuthorityId(3)]);
    pub static FORCE_SESSION_END: RefCell<bool> = RefCell::new(false);
    pub static SESSION_LENGTH: RefCell<u64> = RefCell::new(2);
    pub static SESSION_CHANGED: RefCell<bool> = RefCell::new(false);
    pub static TEST_SESSION_CHANGED: RefCell<bool> = RefCell::new(false);
    pub static DISABLED: RefCell<bool> = RefCell::new(false);
    // Stores if `on_before_session_end` was called
    pub static BEFORE_SESSION_END_CALLED: RefCell<bool> = RefCell::new(false);

}
pub struct TenToFourteen;
impl Contains<u128> for TenToFourteen {
    fn sorted_members() -> Vec<u128> {
        TEN_TO_FOURTEEN.with(|v| v.borrow().clone())
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
impl Contains<u64> for TenToFourteen {
    fn sorted_members() -> Vec<u64> {
        TEN_TO_FOURTEEN.with(|v| v.borrow().clone().iter().map(|x| (*x as u64)).collect())
    }
    #[cfg(feature = "runtime-benchmarks")]
    fn add(new: &u64) {
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
    fn min_len() -> usize {
        0
    }
}
parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: u64 = 1;
    pub const SpendPeriod: u64 = 2;
    pub const Burn: Permill = Permill::from_percent(50);
    pub const TipCountdown: u64 = 1;
    pub const TipFindersFee: Percent = Percent::from_percent(20);
    pub const TipReportDepositBase: u64 = 1;
    pub const DataDepositPerByte: u64 = 1;
    pub const BountyDepositBase: u64 = 80;
    pub const BountyDepositPayoutDelay: u64 = 3;
    pub const TreasuryModuleId: ModuleId = ModuleId(*b"py/trsry");
    pub const BountyUpdatePeriod: u32 = 20;
    pub const MaximumReasonLength: u32 = 16384;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyValueMinimum: u64 = 1;
}
impl pallet_treasury::Trait for Test {
    type ModuleId = TreasuryModuleId;
    type Currency = pallet_balances::Module<Test>;
    type ApproveOrigin = frame_system::EnsureRoot<u64>;
    type RejectOrigin = frame_system::EnsureRoot<u64>;
    type Tippers = TenToFourteen;
    type TipCountdown = TipCountdown;
    type TipFindersFee = TipFindersFee;
    type TipReportDepositBase = TipReportDepositBase;
    type DataDepositPerByte = DataDepositPerByte;
    type Event = ();
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BountyDepositBase = BountyDepositBase;
    type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
    type BountyUpdatePeriod = BountyUpdatePeriod;
    type BountyCuratorDeposit = BountyCuratorDeposit;
    type BountyValueMinimum = BountyValueMinimum;
    type MaximumReasonLength = MaximumReasonLength;
    type BurnDestination = (); // Just gets burned.
    type WeightInfo = ();
}
type System = system::Module<Test>;
type Balances = pallet_balances::Module<Test>;
type Session = pallet_session::Module<Test>;
type Staking = pallet_staking::Module<Test>;

impl<C> frame_system::offchain::SendTransactionTypes<C> for Test
where
    Call: From<C>,
{
    type OverarchingCall = Call;
    type Extrinsic = TestXt<Call, ()>;
}

impl_opaque_keys! {
    pub struct MockSessionKeys {
        pub dummy: UintAuthorityId,
    }
}

pub struct TestShouldEndSession;
impl ShouldEndSession<u64> for TestShouldEndSession {
    fn should_end_session(now: u64) -> bool {
        let l = SESSION_LENGTH.with(|l| *l.borrow());
        now % l == 0
            || FORCE_SESSION_END.with(|l| {
                let r = *l.borrow();
                *l.borrow_mut() = false;
                r
            })
    }
}

pub struct TestSessionHandler;
impl SessionHandler<u64> for TestSessionHandler {
    const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[UintAuthorityId::ID];
    fn on_genesis_session<T: OpaqueKeys>(_validators: &[(u64, T)]) {}
    fn on_new_session<T: OpaqueKeys>(
        changed: bool,
        validators: &[(u64, T)],
        _queued_validators: &[(u64, T)],
    ) {
        SESSION_CHANGED.with(|l| *l.borrow_mut() = changed);
        AUTHORITIES.with(|l| {
            *l.borrow_mut() = validators
                .iter()
                .map(|(_, id)| id.get::<UintAuthorityId>(DUMMY).unwrap_or_default())
                .collect()
        });
    }
    fn on_disabled(_validator_index: usize) {
        DISABLED.with(|l| *l.borrow_mut() = true)
    }
    fn on_before_session_ending() {
        BEFORE_SESSION_END_CALLED.with(|b| *b.borrow_mut() = true);
    }
}

pub struct TestSessionManager;
impl SessionManager<u64> for TestSessionManager {
    fn end_session(_: SessionIndex) {}
    fn start_session(_: SessionIndex) {}
    fn new_session(_: SessionIndex) -> Option<Vec<u64>> {
        if !TEST_SESSION_CHANGED.with(|l| *l.borrow()) {
            VALIDATORS.with(|v| {
                let mut v = v.borrow_mut();
                *v = NEXT_VALIDATORS.with(|l| l.borrow().clone());
                Some(v.clone())
            })
        } else if DISABLED.with(|l| std::mem::replace(&mut *l.borrow_mut(), false)) {
            // If there was a disabled validator, underlying conditions have changed
            // so we return `Some`.
            Some(VALIDATORS.with(|v| v.borrow().clone()))
        } else {
            None
        }
    }
}

impl pallet_session::Trait for Test {
    type Event = ();
    type ValidatorId = <Self as frame_system::Trait>::AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type ShouldEndSession = TestShouldEndSession; //Babe;
    type NextSessionRotation = ();
    type SessionManager = TestSessionManager; //pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type SessionHandler = TestSessionHandler; //<MockSessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = MockSessionKeys;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type WeightInfo = ();
}

impl pallet_session::historical::Trait for Test {
    type FullIdentification = pallet_staking::Exposure<u64, u64>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Self>;
}

parameter_types! {
    pub const UncleGenerations: u64 = 0;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Trait for Test {
    type MaxLocks = ();
    type Balance = u64;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

pallet_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_025_000u64,
        max_inflation: 0_100_000,
        ideal_stake: 0_500_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}

parameter_types! {
    pub const SessionsPerEra: SessionIndex = 3;
    pub const BondingDuration: EraIndex = 3;
    pub const SlashDeferDuration: EraIndex = 0;
    pub const AttestationPeriod: u64 = 100;
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub const ElectionLookahead: u64 = 0;
    pub const StakingUnsignedPriority: u64 = u64::max_value() / 2;
}

pub struct CurrencyToVoteHandler;

impl Convert<u128, u128> for CurrencyToVoteHandler {
    fn convert(x: u128) -> u128 {
        x
    }
}

impl Convert<u128, u64> for CurrencyToVoteHandler {
    fn convert(x: u128) -> u64 {
        x.saturated_into()
    }
}

impl Convert<u64, u64> for CurrencyToVoteHandler {
    fn convert(x: u64) -> u64 {
        x
    }
}

impl pallet_staking::Trait for Test {
    type RewardRemainder = ();
    type CurrencyToVote = CurrencyToVoteHandler;
    type Event = ();
    type Currency = Balances;
    type Slash = ();
    type Reward = ();
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    type SlashCancelOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type SessionInterface = Self;
    type UnixTime = pallet_timestamp::Module<Test>;
    type RewardCurve = RewardCurve;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type NextNewSession = Session;
    type ElectionLookahead = ElectionLookahead;
    type Call = Call;
    type UnsignedPriority = StakingUnsignedPriority;
    type MaxIterations = ();
    type MinSolutionScoreBump = ();
    type WeightInfo = ();
}

parameter_types! {
    pub OffencesWeightSoftLimit: Weight = Perbill::from_percent(60) * MaximumBlockWeight::get();
}

impl pallet_offences::Trait for Test {
    type Event = ();
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
    type WeightSoftLimit = OffencesWeightSoftLimit;
}

impl pallet_social_tokens::Trait for Test {
    type Event = ();
    type Balance = u64;
    type SocialTokenId = u32;
}

impl pallet_social_guardians::Trait for Test {
    type Event = ();
    type SocialId = u32;
}

impl Trait for Test {
    type Event = ();
}

pub type SocialTreasury = Module<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
