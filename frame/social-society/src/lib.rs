#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::traits::{Contains, ContainsLengthBound, EnsureOrigin};
use frame_support::traits::{
    ExistenceRequirement::{AllowDeath, KeepAlive},
    Get, Imbalance, OnUnbalanced, WithdrawReasons,
};
use frame_support::weights::{DispatchClass, Weight};
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, print, Parameter};
use frame_system::{self as system, ensure_signed};
use pallet_staking::EraIndex;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::UniqueSaturatedInto;
use sp_runtime::{
    traits::{BadOrigin, Hash, Saturating, StaticLookup, Zero},
    DispatchResult, Percent, Permill, RuntimeDebug,
};
use sp_std::prelude::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod default_weights;
pub mod weights;

type BalanceOf<T> = <T as pallet_assets::Config>::Balance;
type PositiveImbalanceOf<T> = pallet_assets::PositiveImbalance<T>;
type NegativeImbalanceOf<T> = pallet_assets::NegativeImbalance<T>;
type TokenId<T> = <T as pallet_assets::Config>::AssetId;


pub trait Config:
    frame_system::Config
    + pallet_treasury::Config
    + pallet_staking::Config
    + pallet_assets::Config
    + pallet_social_guardians::Config
{
    /// Origin from which approvals must come.
    type ApproveOrigin: EnsureOrigin<Self::Origin>;

    /// Origin from which rejections must come.
    type RejectOrigin: EnsureOrigin<Self::Origin>;

    /// Origin from which tippers must come.
    ///
    /// `ContainsLengthBound::max_len` must be cost free (i.e. no storage read or heavy operation).
    type Tippers: Contains<Self::AccountId> + ContainsLengthBound;

    /// The period for which a tip remains open after is has achieved threshold tippers.
    type TipCountdown: Get<Self::BlockNumber>;

    /// The percent of the final tip which goes to the original reporter of the tip.
    type TipFindersFee: Get<Percent>;

    /// The amount held on deposit for placing a tip report.
    type TipReportDepositBase: Get<BalanceOf<Self>>;

    /// The amount held on deposit per byte within the tip report reason or bounty description.
    type DataDepositPerByte: Get<BalanceOf<Self>>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    /// Handler for the unbalanced decrease when slashing for a rejected proposal or bounty.
    type OnSlash: OnUnbalanced<NegativeImbalanceOf<Self>>;

    /// Fraction of a proposal's value that should be bonded in order to place the proposal.
    /// An accepted proposal gets these back. A rejected proposal does not.
    type ProposalBond: Get<Permill>;

    /// Minimum amount of funds that should be placed in a deposit for making a proposal.
    type ProposalBondMinimum: Get<BalanceOf<Self>>;

    /// Period between successive spends.
    type SpendPeriod: Get<Self::BlockNumber>;

    /// Percentage of spare funds (if any) that are burnt per spend period.
    type Burn: Get<Permill>;

    /// The amount held on deposit for placing a bounty proposal.
    type BountyDepositBase: Get<BalanceOf<Self>>;

    /// The delay period for which a bounty beneficiary need to wait before claim the payout.
    type BountyDepositPayoutDelay: Get<Self::BlockNumber>;

    /// Bounty duration in blocks.
    type BountyUpdatePeriod: Get<Self::BlockNumber>;

    /// Percentage of the curator fee that will be reserved upfront as deposit for bounty curator.
    type BountyCuratorDeposit: Get<Permill>;

    /// Minimum value for a bounty.
    type BountyValueMinimum: Get<BalanceOf<Self>>;

    /// Maximum acceptable reason length.
    type MaximumReasonLength: Get<u32>;

    /// Handler for the unbalanced decrease when treasury funds are burned.
    type BurnDestination: OnUnbalanced<NegativeImbalanceOf<Self>>;

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
}

/// An index of a proposal. Just a `u32`.
pub type ProposalIndex = u32;

/// A spending proposal.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Proposal<AccountId, Balance, AssetId> {
    /// The account proposing it.
    proposer: AccountId,
    /// The (total) amount that should be paid if the proposal is accepted.
    value: Balance,
    /// The account to whom the payment should be made if the proposal is accepted.
    beneficiary: AccountId,
    /// The amount held on deposit (reserved) for making this proposal.
    bond: Balance,
    social_token_id: AssetId,
}

/// An open tipping "motion". Retains all details of a tip including information on the finder
/// and the members who have voted.
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct OpenTip<
    AccountId: Parameter,
    Balance: Parameter,
    BlockNumber: Parameter,
    Hash: Parameter,
    AssetId: Parameter,
> {
    /// The hash of the reason for the tip. The reason should be a human-readable UTF-8 encoded string. A URL would be
    /// sensible.
    reason: Hash,
    /// The account to be tipped.
    who: AccountId,
    /// The account who began this tip.
    finder: AccountId,
    /// The amount held on deposit for this tip.
    deposit: Balance,
    /// The block number at which this tip will close if `Some`. If `None`, then no closing is
    /// scheduled.
    closes: Option<BlockNumber>,
    /// The members who have voted for this tip. Sorted by AccountId.
    tips: Vec<(AccountId, Balance)>,
    /// Whether this tip should result in the finder taking a fee.
    finders_fee: bool,
    social_token_id: AssetId,
}

/// An index of a bounty. Just a `u32`.
pub type BountyIndex = u32;

/// A bounty proposal.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Bounty<AccountId, Balance, BlockNumber, AssetId> {
    /// The account proposing it.
    proposer: AccountId,
    /// The (total) amount that should be paid if the bounty is rewarded.
    value: Balance,
    /// The curator fee. Included in value.
    fee: Balance,
    /// The deposit of curator.
    curator_deposit: Balance,
    /// The amount held on deposit (reserved) for making this proposal.
    bond: Balance,
    /// The status of this bounty.
    status: BountyStatus<AccountId, BlockNumber>,
    social_token_id: AssetId,
}

/// The status of a bounty proposal.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum BountyStatus<AccountId, BlockNumber> {
    /// The bounty is proposed and waiting for approval.
    Proposed,
    /// The bounty is approved and waiting to become active at next spend period.
    Approved,
    /// The bounty is funded and waiting for curator assignment.
    Funded,
    /// A curator has been proposed by the `ApproveOrigin`. Waiting for acceptance from the curator.
    CuratorProposed {
        /// The assigned curator of this bounty.
        curator: AccountId,
    },
    /// The bounty is active and waiting to be awarded.
    Active {
        /// The curator of this bounty.
        curator: AccountId,
        /// An update from the curator is due by this block, else they are considered inactive.
        update_due: BlockNumber,
    },
    /// The bounty is awarded and waiting to released after a delay.
    PendingPayout {
        /// The curator of this bounty.
        curator: AccountId,
        /// The beneficiary of the bounty.
        beneficiary: AccountId,
        /// When the bounty can be claimed.
        unlock_at: BlockNumber,
    },
}

decl_storage! {
    trait Store for Module<T: Config> as SocialTreasury {
        Something get(fn something): Option<u32>;
        NextEraForProcessing get(fn next_era_for_processing): Option<EraIndex>;

        /// Number of proposals that have been made.
        ProposalCount get(fn proposal_count): ProposalIndex;

        /// Proposals that have been made.
        Proposals get(fn proposals):
            map hasher(twox_64_concat) ProposalIndex
            => Option<Proposal<T::AccountId, BalanceOf<T>, TokenId<T>>>;

        /// Proposal indices that have been approved but not yet awarded.
        Approvals get(fn approvals): Vec<ProposalIndex>;

        /// Tips that are not yet completed. Keyed by the hash of `(reason, who)` from the value.
        /// This has the insecure enumerable hash function since the key itself is already
        /// guaranteed to be a secure hash.
        pub Tips get(fn tips):
            map hasher(twox_64_concat) T::Hash
            => Option<OpenTip<T::AccountId, BalanceOf<T>, T::BlockNumber, T::Hash, TokenId<T>>>;

        /// Simple preimage lookup from the reason's hash to the original data. Again, has an
        /// insecure enumerable hash since the key is guaranteed to be the result of a secure hash.
        pub Reasons get(fn reasons): map hasher(identity) T::Hash => Option<Vec<u8>>;

        /// Number of bounty proposals that have been made.
        pub BountyCount get(fn bounty_count): BountyIndex;

        /// Bounties that have been made.
        pub Bounties get(fn bounties):
            map hasher(twox_64_concat) BountyIndex
            => Option<Bounty<T::AccountId, BalanceOf<T>, T::BlockNumber, TokenId<T>>>;

        /// The description of each bounty.
        pub BountyDescriptions get(fn bounty_descriptions): map hasher(twox_64_concat) BountyIndex => Option<Vec<u8>>;

        /// Bounty indices that have been approved but not yet funded.
        pub BountyApprovals get(fn bounty_approvals): Vec<BountyIndex>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Hash = <T as frame_system::Config>::Hash,
        SocialTokenBalance = <T as pallet_assets::Config>::Balance,
        AssetId = <T as pallet_assets::Config>::AssetId,
    {
        /// New proposal. \[proposal_index\]
        Proposed(ProposalIndex),
        /// We have ended a spend period and will now allocate funds. \[budget_remaining\]
        Spending(AssetId, SocialTokenBalance),
        /// Some funds have been allocated. \[proposal_index, award, beneficiary\]
        Awarded(ProposalIndex, AssetId, SocialTokenBalance, AccountId),
        /// A proposal was rejected; funds were slashed. \[proposal_index, slashed\]
        Rejected(ProposalIndex, AssetId, SocialTokenBalance),
        /// Some of our funds have been burnt. \[burn\]
        Burnt(AssetId, SocialTokenBalance),
        /// Spending has finished; this is the amount that rolls over until next spend.
        /// \[budget_remaining\]
        Rollover(AssetId, SocialTokenBalance),
        /// Some funds have been deposited. \[deposit\]
        Deposit(AssetId, SocialTokenBalance),
        /// A new tip suggestion has been opened. \[tip_hash\]
        NewTip(Hash),
        /// A tip suggestion has reached threshold and is closing. \[tip_hash\]
        TipClosing(Hash),
        /// A tip suggestion has been closed. \[tip_hash, who, payout\]
        TipClosed(Hash, AccountId, AssetId, SocialTokenBalance),
        /// A tip suggestion has been retracted. \[tip_hash\]
        TipRetracted(Hash),
        /// New bounty proposal. [index]
        BountyProposed(BountyIndex),
        /// A bounty proposal was rejected; funds were slashed. [index, bond]
        BountyRejected(BountyIndex, AssetId, SocialTokenBalance),
        /// A bounty proposal is funded and became active. [index]
        BountyBecameActive(BountyIndex),
        /// A bounty is awarded to a beneficiary. [index, beneficiary]
        BountyAwarded(BountyIndex, AccountId),
        /// A bounty is claimed by beneficiary. [index, payout, beneficiary]
        BountyClaimed(BountyIndex, AssetId, SocialTokenBalance, AccountId),
        /// A bounty is cancelled. [index]
        BountyCanceled(BountyIndex),
        /// A bounty expiry is extended. [index]
        BountyExtended(BountyIndex),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,

        /// Proposer's balance is too low.
        InsufficientProposersBalance,
        /// No proposal or bounty at that index.
        InvalidIndex,
        /// The reason given is just too big.
        ReasonTooBig,
        /// The tip was already found/started.
        AlreadyKnown,
        /// The tip hash is unknown.
        UnknownTip,
        /// The account attempting to retract the tip is not the finder of the tip.
        NotFinder,
        /// The tip cannot be claimed/closed because there are not enough tippers yet.
        StillOpen,
        /// The tip cannot be claimed/closed because it's still in the countdown period.
        Premature,
        /// The bounty status is unexpected.
        UnexpectedStatus,
        /// Require bounty curator.
        RequireCurator,
        /// Invalid bounty value.
        InvalidValue,
        /// Invalid bounty fee.
        InvalidFee,
        /// A bounty payout is pending.
        /// To cancel the bounty, you must unassign and slash the curator.
        PendingPayout,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Put forward a suggestion for spending. A deposit proportional to the value
        /// is reserved and slashed if the proposal is rejected. It is returned once the
        /// proposal is awarded.
        ///
        /// # <weight>
        /// - Complexity: O(1)
        /// - DbReads: `ProposalCount`, `origin account`
        /// - DbWrites: `ProposalCount`, `Proposals`, `origin account`
        /// # </weight>
        #[weight = <T as Config>::WeightInfo::propose_spend()]
        fn propose_spend(
            origin,
            #[compact] value: BalanceOf<T>,
            beneficiary: <T::Lookup as StaticLookup>::Source,
            token_id: TokenId<T>,
        ) {
            let proposer = ensure_signed(origin)?;
            let beneficiary = T::Lookup::lookup(beneficiary)?;
            // <pallet_assets::Module<T>>::validate_social_token_id(token_id)?;

            let bond = Self::calculate_bond(value);
            // <pallet_assets::Module<T>>::reserve(&proposer, token_id, bond)
            //     .map_err(|_| Error::<T>::InsufficientProposersBalance)?;

            let c = Self::proposal_count();
            <ProposalCount>::put(c + 1);
            <Proposals<T>>::insert(c, Proposal { proposer, value, beneficiary, bond, social_token_id: token_id });

            Self::deposit_event(RawEvent::Proposed(c));
        }

        /// Reject a proposed spend. The original deposit will be slashed.
        ///
        /// May only be called from `T::RejectOrigin`.
        ///
        /// # <weight>
        /// - Complexity: O(1)
        /// - DbReads: `Proposals`, `rejected proposer account`
        /// - DbWrites: `Proposals`, `rejected proposer account`
        /// # </weight>
        #[weight = (<T as Config>::WeightInfo::reject_proposal(), DispatchClass::Operational)]
        fn reject_proposal(origin, #[compact] proposal_id: ProposalIndex) {
            <T as Config>::RejectOrigin::ensure_origin(origin)?;

            let proposal = <Proposals<T>>::take(&proposal_id).ok_or(Error::<T>::InvalidIndex)?;
            let value = proposal.bond;
            // let imbalance = <pallet_assets::Module<T>>::slash_reserved(
            //     &proposal.proposer,
            //     proposal.social_token_id,
            //     value
            // ).0;
            // <T as Config>::OnSlash::on_unbalanced(imbalance);

            Self::deposit_event(Event::<T>::Rejected(proposal_id, proposal.social_token_id, value));
        }

        /// Approve a proposal. At a later time, the proposal will be allocated to the beneficiary
        /// and the original deposit will be returned.
        ///
        /// May only be called from `T::ApproveOrigin`.
        ///
        /// # <weight>
        /// - Complexity: O(1).
        /// - DbReads: `Proposals`, `Approvals`
        /// - DbWrite: `Approvals`
        /// # </weight>
        #[weight = (<T as Config>::WeightInfo::approve_proposal(), DispatchClass::Operational)]
        fn approve_proposal(origin, #[compact] proposal_id: ProposalIndex) {
            <T as Config>::ApproveOrigin::ensure_origin(origin)?;

            ensure!(<Proposals<T>>::contains_key(proposal_id), Error::<T>::InvalidIndex);
            Approvals::append(proposal_id);
        }

        /// Report something `reason` that deserves a tip and claim any eventual the finder's fee.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// Payment: `TipReportDepositBase` will be reserved from the origin account, as well as
        /// `DataDepositPerByte` for each byte in `reason`.
        ///
        /// - `reason`: The reason for, or the thing that deserves, the tip; generally this will be
        ///   a UTF-8-encoded URL.
        /// - `who`: The account which should be credited for the tip.
        ///
        /// Emits `NewTip` if successful.
        ///
        /// # <weight>
        /// - Complexity: `O(R)` where `R` length of `reason`.
        ///   - encoding and hashing of 'reason'
        /// - DbReads: `Reasons`, `Tips`
        /// - DbWrites: `Reasons`, `Tips`
        /// # </weight>
        #[weight = <T as Config>::WeightInfo::report_awesome(reason.len() as u32)]
        fn report_awesome(origin, reason: Vec<u8>, who: T::AccountId, token_id: TokenId<T>) {
            let finder = ensure_signed(origin)?;

            ensure!(reason.len() <= <T as Config>::MaximumReasonLength::get() as usize, Error::<T>::ReasonTooBig);
            // <pallet_assets::Module<T>>::validate_social_token_id(token_id)?;

            let reason_hash = <T as frame_system::Config>::Hashing::hash(&reason[..]);
            ensure!(!Reasons::<T>::contains_key(&reason_hash), Error::<T>::AlreadyKnown);
            let hash = <T as frame_system::Config>::Hashing::hash_of(&(&reason_hash, &who));
            ensure!(!Tips::<T>::contains_key(&hash), Error::<T>::AlreadyKnown);

            let deposit = <T as Config>::TipReportDepositBase::get()
                + <T as Config>::DataDepositPerByte::get() * (reason.len() as u32).into();
            // <pallet_assets::Module<T>>::reserve(&finder, token_id, deposit)?;

            Reasons::<T>::insert(&reason_hash, &reason);
            let tip = OpenTip {
                reason: reason_hash,
                who,
                finder,
                deposit,
                closes: None,
                tips: vec![],
                finders_fee: true,
                social_token_id: token_id,
            };
            Tips::<T>::insert(&hash, tip);
            Self::deposit_event(RawEvent::NewTip(hash));
        }

        /// Retract a prior tip-report from `report_awesome`, and cancel the process of tipping.
        ///
        /// If successful, the original deposit will be unreserved.
        ///
        /// The dispatch origin for this call must be _Signed_ and the tip identified by `hash`
        /// must have been reported by the signing account through `report_awesome` (and not
        /// through `tip_new`).
        ///
        /// - `hash`: The identity of the open tip for which a tip value is declared. This is formed
        ///   as the hash of the tuple of the original tip `reason` and the beneficiary account ID.
        ///
        /// Emits `TipRetracted` if successful.
        ///
        /// # <weight>
        /// - Complexity: `O(1)`
        ///   - Depends on the length of `T::Hash` which is fixed.
        /// - DbReads: `Tips`, `origin account`
        /// - DbWrites: `Reasons`, `Tips`, `origin account`
        /// # </weight>
        #[weight = <T as Config>::WeightInfo::retract_tip()]
        fn retract_tip(origin, hash: T::Hash) {
            let who = ensure_signed(origin)?;
            let tip = Tips::<T>::get(&hash).ok_or(Error::<T>::UnknownTip)?;
            ensure!(tip.finder == who, Error::<T>::NotFinder);

            Reasons::<T>::remove(&tip.reason);
            Tips::<T>::remove(&hash);
            // if !tip.deposit.is_zero() {
            //     let _ = <pallet_assets::Module<T>>::unreserve(&who, tip.social_token_id, tip.deposit);
            // }
            Self::deposit_event(RawEvent::TipRetracted(hash));
        }

        /// Give a tip for something new; no finder's fee will be taken.
        ///
        /// The dispatch origin for this call must be _Signed_ and the signing account must be a
        /// member of the `Tippers` set.
        ///
        /// - `reason`: The reason for, or the thing that deserves, the tip; generally this will be
        ///   a UTF-8-encoded URL.
        /// - `who`: The account which should be credited for the tip.
        /// - `tip_value`: The amount of tip that the sender would like to give. The median tip
        ///   value of active tippers will be given to the `who`.
        ///
        /// Emits `NewTip` if successful.
        ///
        /// # <weight>
        /// - Complexity: `O(R + T)` where `R` length of `reason`, `T` is the number of tippers.
        ///   - `O(T)`: decoding `Tipper` vec of length `T`
        ///     `T` is charged as upper bound given by `ContainsLengthBound`.
        ///     The actual cost depends on the implementation of `T::Tippers`.
        ///   - `O(R)`: hashing and encoding of reason of length `R`
        /// - DbReads: `Tippers`, `Reasons`
        /// - DbWrites: `Reasons`, `Tips`
        /// # </weight>
        #[weight = <T as Config>::WeightInfo::tip_new(reason.len() as u32, <T as Config>::Tippers::max_len() as u32)]
        fn tip_new(origin, reason: Vec<u8>, who: T::AccountId, #[compact] token_id: TokenId<T>, #[compact] tip_value: BalanceOf<T>) {
            let tipper = ensure_signed(origin)?;
            // <pallet_assets::Module<T>>::validate_social_token_id(token_id)?;
            ensure!(<T as Config>::Tippers::contains(&tipper), BadOrigin);
            let reason_hash = <T as frame_system::Config>::Hashing::hash(&reason[..]);
            ensure!(!Reasons::<T>::contains_key(&reason_hash), Error::<T>::AlreadyKnown);
            let hash = <T as frame_system::Config>::Hashing::hash_of(&(&reason_hash, &who));

            Reasons::<T>::insert(&reason_hash, &reason);
            Self::deposit_event(RawEvent::NewTip(hash.clone()));
            let tips = vec![(tipper.clone(), tip_value)];
            let tip = OpenTip {
                reason: reason_hash,
                who,
                finder: tipper,
                deposit: Zero::zero(),
                closes: None,
                tips,
                finders_fee: false,
                social_token_id: token_id,
            };
            Tips::<T>::insert(&hash, tip);
        }

        /// Declare a tip value for an already-open tip.
        ///
        /// The dispatch origin for this call must be _Signed_ and the signing account must be a
        /// member of the `Tippers` set.
        ///
        /// - `hash`: The identity of the open tip for which a tip value is declared. This is formed
        ///   as the hash of the tuple of the hash of the original tip `reason` and the beneficiary
        ///   account ID.
        /// - `tip_value`: The amount of tip that the sender would like to give. The median tip
        ///   value of active tippers will be given to the `who`.
        ///
        /// Emits `TipClosing` if the threshold of tippers has been reached and the countdown period
        /// has started.
        ///
        /// # <weight>
        /// - Complexity: `O(T)` where `T` is the number of tippers.
        ///   decoding `Tipper` vec of length `T`, insert tip and check closing,
        ///   `T` is charged as upper bound given by `ContainsLengthBound`.
        ///   The actual cost depends on the implementation of `T::Tippers`.
        ///
        ///   Actually weight could be lower as it depends on how many tips are in `OpenTip` but it
        ///   is weighted as if almost full i.e of length `T-1`.
        /// - DbReads: `Tippers`, `Tips`
        /// - DbWrites: `Tips`
        /// # </weight>
        #[weight = <T as Config>::WeightInfo::tip(<T as Config>::Tippers::max_len() as u32)]
        fn tip(origin, hash: T::Hash, #[compact] tip_value: BalanceOf<T>) {
            let tipper = ensure_signed(origin)?;
            ensure!(<T as Config>::Tippers::contains(&tipper), BadOrigin);

            let mut tip = Tips::<T>::get(hash).ok_or(Error::<T>::UnknownTip)?;
            if Self::insert_tip_and_check_closing(&mut tip, tipper, tip_value) {
                Self::deposit_event(RawEvent::TipClosing(hash.clone()));
            }
            Tips::<T>::insert(&hash, tip);
        }

        /// Close and payout a tip.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// The tip identified by `hash` must have finished its countdown period.
        ///
        /// - `hash`: The identity of the open tip for which a tip value is declared. This is formed
        ///   as the hash of the tuple of the original tip `reason` and the beneficiary account ID.
        ///
        /// # <weight>
        /// - Complexity: `O(T)` where `T` is the number of tippers.
        ///   decoding `Tipper` vec of length `T`.
        ///   `T` is charged as upper bound given by `ContainsLengthBound`.
        ///   The actual cost depends on the implementation of `T::Tippers`.
        /// - DbReads: `Tips`, `Tippers`, `tip finder`
        /// - DbWrites: `Reasons`, `Tips`, `Tippers`, `tip finder`
        /// # </weight>
        #[weight = <T as Config>::WeightInfo::close_tip(<T as Config>::Tippers::max_len() as u32)]
        fn close_tip(origin, hash: T::Hash) {
            ensure_signed(origin)?;

            let tip = Tips::<T>::get(hash).ok_or(Error::<T>::UnknownTip)?;
            let n = tip.closes.as_ref().ok_or(Error::<T>::StillOpen)?;
            ensure!(system::Module::<T>::block_number() >= *n, Error::<T>::Premature);
            // closed.
            Reasons::<T>::remove(&tip.reason);
            Tips::<T>::remove(hash);
            Self::payout_tip(hash, tip);
        }

        /// Propose a new bounty.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// Payment: `TipReportDepositBase` will be reserved from the origin account, as well as
        /// `DataDepositPerByte` for each byte in `reason`. It will be unreserved upon approval,
        /// or slashed when rejected.
        ///
        /// - `curator`: The curator account whom will manage this bounty.
        /// - `fee`: The curator fee.
        /// - `value`: The total payment amount of this bounty, curator fee included.
        /// - `description`: The description of this bounty.
        #[weight = <T as Config>::WeightInfo::propose_bounty(description.len() as u32)]
        fn propose_bounty(
            origin,
            #[compact] value: BalanceOf<T>,
            description: Vec<u8>,
            token_id: TokenId<T>,
        ) {
            let proposer = ensure_signed(origin)?;
            // <pallet_assets::Module<T>>::validate_social_token_id(token_id)?;
            Self::create_bounty(proposer, description, token_id, value)?;
        }

        /// Approve a bounty proposal. At a later time, the bounty will be funded and become active
        /// and the original deposit will be returned.
        ///
        /// May only be called from `T::ApproveOrigin`.
        ///
        /// # <weight>
        /// - O(1).
        /// - Limited storage reads.
        /// - One DB change.
        /// # </weight>
        #[weight = <T as Config>::WeightInfo::approve_bounty()]
        fn approve_bounty(origin, #[compact] bounty_id: ProposalIndex) {
            <T as Config>::ApproveOrigin::ensure_origin(origin)?;

            Bounties::<T>::try_mutate_exists(bounty_id, |maybe_bounty| -> DispatchResult {
                let mut bounty = maybe_bounty.as_mut().ok_or(Error::<T>::InvalidIndex)?;
                ensure!(bounty.status == BountyStatus::Proposed, Error::<T>::UnexpectedStatus);

                bounty.status = BountyStatus::Approved;

                BountyApprovals::append(bounty_id);

                Ok(())
            })?;
        }

        /// Assign a curator to a funded bounty.
        ///
        /// May only be called from `T::ApproveOrigin`.
        ///
        /// # <weight>
        /// - O(1).
        /// - Limited storage reads.
        /// - One DB change.
        /// # </weight>
        #[weight = <T as Config>::WeightInfo::propose_curator()]
        fn propose_curator(
            origin,
            #[compact] bounty_id: ProposalIndex,
            curator: <T::Lookup as StaticLookup>::Source,
            #[compact] fee: BalanceOf<T>,
        ) {
            <T as Config>::ApproveOrigin::ensure_origin(origin)?;

            let curator = T::Lookup::lookup(curator)?;
            Bounties::<T>::try_mutate_exists(bounty_id, |maybe_bounty| -> DispatchResult {
                let mut bounty = maybe_bounty.as_mut().ok_or(Error::<T>::InvalidIndex)?;
                match bounty.status {
                    BountyStatus::Funded | BountyStatus::CuratorProposed { .. } => {},
                    _ => return Err(Error::<T>::UnexpectedStatus.into()),
                };

                ensure!(fee < bounty.value, Error::<T>::InvalidFee);

                bounty.status = BountyStatus::CuratorProposed { curator };
                bounty.fee = fee;

                Ok(())
            })?;
        }

        /// Unassign curator from a bounty.
        ///
        /// This function can only be called by the `RejectOrigin` a signed origin.
        ///
        /// If this function is called by the `RejectOrigin`, we assume that the curator is malicious
        /// or inactive. As a result, we will slash the curator when possible.
        ///
        /// If the origin is the curator, we take this as a sign they are unable to do their job and
        /// they willingly give up. We could slash them, but for now we allow them to recover their
        /// deposit and exit without issue. (We may want to change this if it is abused.)
        ///
        /// Finally, the origin can be anyone if and only if the curator is "inactive". This allows
        /// anyone in the community to call out that a curator is not doing their due diligence, and
        /// we should pick a new curator. In this case the curator should also be slashed.
        ///
        /// # <weight>
        /// - O(1).
        /// - Limited storage reads.
        /// - One DB change.
        /// # </weight>
        #[weight = <T as Config>::WeightInfo::unassign_curator()]
        fn unassign_curator(
            origin,
            #[compact] bounty_id: ProposalIndex,
        ) {
            let maybe_sender = ensure_signed(origin.clone())
                .map(Some)
                .or_else(|_| <T as Config>::RejectOrigin::ensure_origin(origin).map(|_| None))?;

            Bounties::<T>::try_mutate_exists(bounty_id, |maybe_bounty| -> DispatchResult {
                let mut bounty = maybe_bounty.as_mut().ok_or(Error::<T>::InvalidIndex)?;
                let token_id = bounty.social_token_id;

                let slash_curator = |curator: &T::AccountId, curator_deposit: &mut BalanceOf<T>| {
                    // let imbalance = <pallet_assets::Module<T>>::slash_reserved(curator, token_id, *curator_deposit).0;
                    // <T as Config>::OnSlash::on_unbalanced(imbalance);
                    *curator_deposit = Zero::zero();
                };

                match bounty.status {
                    BountyStatus::Proposed | BountyStatus::Approved | BountyStatus::Funded => {
                        // No curator to unassign at this point.
                        return Err(Error::<T>::UnexpectedStatus.into())
                    }
                    BountyStatus::CuratorProposed { ref curator } => {
                        // A curator has been proposed, but not accepted yet.
                        // Either `RejectOrigin` or the proposed curator can unassign the curator.
                        ensure!(maybe_sender.map_or(true, |sender| sender == *curator), BadOrigin);
                    },
                    BountyStatus::Active { ref curator, ref update_due } => {
                        // The bounty is active.
                        match maybe_sender {
                            // If the `RejectOrigin` is calling this function, slash the curator.
                            None => {
                                slash_curator(curator, &mut bounty.curator_deposit);
                                // Continue to change bounty status below...
                            },
                            Some(sender) => {
                                // If the sender is not the curator, and the curator is inactive,
                                // slash the curator.
                                if sender != *curator {
                                    let block_number = system::Module::<T>::block_number();
                                    if *update_due < block_number {
                                        slash_curator(curator, &mut bounty.curator_deposit);
                                        // Continue to change bounty status below...
                                    } else {
                                        // Curator has more time to give an update.
                                        return Err(Error::<T>::Premature.into())
                                    }
                                } else {
                                    // Else this is the curator, willingly giving up their role.
                                    // Give back their deposit.
                                    // let _ = <pallet_assets::Module<T>>::unreserve(&curator, bounty.social_token_id, bounty.curator_deposit);
                                    // Continue to change bounty status below...
                                }
                            },
                        }
                    },
                    BountyStatus::PendingPayout { ref curator, .. } => {
                        // The bounty is pending payout, so only council can unassign a curator.
                        // By doing so, they are claiming the curator is acting maliciously, so
                        // we slash the curator.
                        ensure!(maybe_sender.is_none(), BadOrigin);
                        slash_curator(curator, &mut bounty.curator_deposit);
                        // Continue to change bounty status below...
                    }
                };

                bounty.status = BountyStatus::Funded;
                Ok(())
            })?;
        }

        /// Accept the curator role for a bounty.
        /// A deposit will be reserved from curator and refund upon successful payout.
        ///
        /// May only be called from the curator.
        ///
        /// # <weight>
        /// - O(1).
        /// - Limited storage reads.
        /// - One DB change.
        /// # </weight>
        #[weight = <T as Config>::WeightInfo::accept_curator()]
        fn accept_curator(origin, #[compact] bounty_id: ProposalIndex) {
            let signer = ensure_signed(origin)?;

            Bounties::<T>::try_mutate_exists(bounty_id, |maybe_bounty| -> DispatchResult {
                let mut bounty = maybe_bounty.as_mut().ok_or(Error::<T>::InvalidIndex)?;

                match bounty.status {
                    BountyStatus::CuratorProposed { ref curator } => {
                        ensure!(signer == *curator, Error::<T>::RequireCurator);

                        let deposit = <T as Config>::BountyCuratorDeposit::get() * bounty.fee;
                        // <pallet_assets::Module<T>>::reserve(curator, bounty.social_token_id, deposit)?;
                        bounty.curator_deposit = deposit;

                        let update_due = system::Module::<T>::block_number() + <T as Config>::BountyUpdatePeriod::get();
                        bounty.status = BountyStatus::Active { curator: curator.clone(), update_due };

                        Ok(())
                    },
                    _ => Err(Error::<T>::UnexpectedStatus.into()),
                }
            })?;
        }

        /// Award bounty to a beneficiary account. The beneficiary will be able to claim the funds after a delay.
        ///
        /// The dispatch origin for this call must be the curator of this bounty.
        ///
        /// - `bounty_id`: Bounty ID to award.
        /// - `beneficiary`: The beneficiary account whom will receive the payout.
        #[weight = <T as Config>::WeightInfo::award_bounty()]
        fn award_bounty(origin, #[compact] bounty_id: ProposalIndex, beneficiary: <T::Lookup as StaticLookup>::Source) {
            let signer = ensure_signed(origin)?;
            let beneficiary = T::Lookup::lookup(beneficiary)?;

            Bounties::<T>::try_mutate_exists(bounty_id, |maybe_bounty| -> DispatchResult {
                let mut bounty = maybe_bounty.as_mut().ok_or(Error::<T>::InvalidIndex)?;
                match &bounty.status {
                    BountyStatus::Active {
                        curator,
                        ..
                    } => {
                        ensure!(signer == *curator, Error::<T>::RequireCurator);
                    },
                    _ => return Err(Error::<T>::UnexpectedStatus.into()),
                }
                bounty.status = BountyStatus::PendingPayout {
                    curator: signer,
                    beneficiary: beneficiary.clone(),
                    unlock_at: system::Module::<T>::block_number() + <T as Config>::BountyDepositPayoutDelay::get(),
                };

                Ok(())
            })?;

            Self::deposit_event(Event::<T>::BountyAwarded(bounty_id, beneficiary));
        }

        /// Claim the payout from an awarded bounty after payout delay.
        ///
        /// The dispatch origin for this call must be the beneficiary of this bounty.
        ///
        /// - `bounty_id`: Bounty ID to claim.
        #[weight = <T as Config>::WeightInfo::claim_bounty()]
        fn claim_bounty(origin, #[compact] bounty_id: BountyIndex) {
            let _ = ensure_signed(origin)?; // anyone can trigger claim

            Bounties::<T>::try_mutate_exists(bounty_id, |maybe_bounty| -> DispatchResult {
                let bounty = maybe_bounty.take().ok_or(Error::<T>::InvalidIndex)?;
                if let BountyStatus::PendingPayout { curator, beneficiary, unlock_at } = bounty.status {
                    ensure!(system::Module::<T>::block_number() >= unlock_at, Error::<T>::Premature);
                    let bounty_account = Self::bounty_account_id(bounty_id);
                    // let balance = <pallet_assets::Module<T>>::free_balance(&bounty_account, bounty.social_token_id);
                    // let fee = bounty.fee; // just to be safe
                    // let payout = balance.saturating_sub(fee);
                    let payout = 0u32.into();
                    // let _ = <pallet_assets::Module<T>>::unreserve(&curator, bounty.social_token_id, bounty.curator_deposit);
                    // let _ = <pallet_assets::Module<T>>::do_transfer(&bounty_account, &curator, bounty.social_token_id, fee, AllowDeath); // should not fail
                    // let _ = <pallet_assets::Module<T>>::do_transfer(&bounty_account, &beneficiary, bounty.social_token_id, payout, AllowDeath); // should not fail
                    *maybe_bounty = None;

                    BountyDescriptions::remove(bounty_id);

                    Self::deposit_event(Event::<T>::BountyClaimed(bounty_id, bounty.social_token_id, payout, beneficiary));
                    Ok(())
                } else {
                    Err(Error::<T>::UnexpectedStatus.into())
                }
            })?;
        }

        /// Cancel a proposed or active bounty. All the funds will be sent to treasury and
        /// the curator deposit will be unreserved if possible.
        ///
        /// Only `T::RejectOrigin` is able to cancel a bounty.
        ///
        /// - `bounty_id`: Bounty ID to cancel.
        #[weight = <T as Config>::WeightInfo::close_bounty_proposed().max(<T as Config>::WeightInfo::close_bounty_active())]
        fn close_bounty(origin, #[compact] bounty_id: BountyIndex) -> DispatchResultWithPostInfo {
            <T as Config>::RejectOrigin::ensure_origin(origin)?;

            Bounties::<T>::try_mutate_exists(bounty_id, |maybe_bounty| -> DispatchResultWithPostInfo {
                let bounty = maybe_bounty.as_ref().ok_or(Error::<T>::InvalidIndex)?;
                let token_id = bounty.social_token_id;

                match &bounty.status {
                    BountyStatus::Proposed => {
                        // The reject origin would like to cancel a proposed bounty.
                        BountyDescriptions::remove(bounty_id);
                        let value = bounty.bond;
                        // let imbalance = <pallet_assets::Module<T>>::slash_reserved(&bounty.proposer, token_id, value).0;
                        // <T as Config>::OnSlash::on_unbalanced(imbalance);
                        *maybe_bounty = None;

                        Self::deposit_event(Event::<T>::BountyRejected(bounty_id, token_id, value));
                        // Return early, nothing else to do.
                        return Ok(Some(<T as Config>::WeightInfo::close_bounty_proposed()).into())
                    },
                    BountyStatus::Approved => {
                        // For weight reasons, we don't allow a council to cancel in this phase.
                        // We ask for them to wait until it is funded before they can cancel.
                        return Err(Error::<T>::UnexpectedStatus.into())
                    },
                    BountyStatus::Funded |
                    BountyStatus::CuratorProposed { .. } => {
                        // Nothing extra to do besides the removal of the bounty below.
                    },
                    BountyStatus::Active { curator, .. } => {
                        // Cancelled by council, refund deposit of the working curator.
                        // let _ = <pallet_assets::Module<T>>::unreserve(&curator, token_id, bounty.curator_deposit);
                        // Then execute removal of the bounty below.
                    },
                    BountyStatus::PendingPayout { .. } => {
                        // Bounty is already pending payout. If council wants to cancel
                        // this bounty, it should mean the curator was acting maliciously.
                        // So the council should first unassign the curator, slashing their
                        // deposit.
                        return Err(Error::<T>::PendingPayout.into())
                    }
                }

                let bounty_account = Self::bounty_account_id(bounty_id);

                BountyDescriptions::remove(bounty_id);

                // let balance = <pallet_assets::Module<T>>::free_balance(&bounty_account, bounty.social_token_id);
                // let _ = <pallet_assets::Module<T>>::do_transfer(
                //     &bounty_account,
                //     &Self::account_id(),
                //     bounty.social_token_id,
                //     balance,
                //     AllowDeath
                // ); // should not fail
                *maybe_bounty = None;

                Self::deposit_event(Event::<T>::BountyCanceled(bounty_id));
                Ok(Some(<T as Config>::WeightInfo::close_bounty_active()).into())
            })
        }

        /// Extend the expiry time of an active bounty.
        ///
        /// The dispatch origin for this call must be the curator of this bounty.
        ///
        /// - `bounty_id`: Bounty ID to extend.
        /// - `remark`: additional information.
        #[weight = <T as Config>::WeightInfo::extend_bounty_expiry()]
        fn extend_bounty_expiry(origin, #[compact] bounty_id: BountyIndex, _remark: Vec<u8>) {
            let signer = ensure_signed(origin)?;

            Bounties::<T>::try_mutate_exists(bounty_id, |maybe_bounty| -> DispatchResult {
                let bounty = maybe_bounty.as_mut().ok_or(Error::<T>::InvalidIndex)?;

                match bounty.status {
                    BountyStatus::Active { ref curator, ref mut update_due } => {
                        ensure!(*curator == signer, Error::<T>::RequireCurator);
                        *update_due = (system::Module::<T>::block_number() + <T as Config>::BountyUpdatePeriod::get()).max(*update_due);
                    },
                    _ => return Err(Error::<T>::UnexpectedStatus.into()),
                }

                Ok(())
            })?;

            Self::deposit_event(Event::<T>::BountyExtended(bounty_id));
        }

        /// # <weight>
        /// - Complexity: `O(A)` where `A` is the number of approvals
        /// - Db reads and writes: `Approvals`, `pot account data`
        /// - Db reads and writes per approval:
        ///   `Proposals`, `proposer account data`, `beneficiary account data`
        /// - The weight is overestimated if some approvals got missed.
        /// # </weight>
        fn on_initialize(n: T::BlockNumber) -> Weight {
            // Check to see if we should spend some funds!
            if (n % <T as Config>::SpendPeriod::get()).is_zero() {
                Self::spend_funds()
            } else {
                0
            }
        }

        fn on_finalize() {
            match <pallet_staking::Module<T>>::current_era() {
                None => (),
                Some(current_era) => {
                    let era = NextEraForProcessing::get().unwrap_or(0);
                    if era < current_era {
                        let reward_points = <pallet_staking::Module<T>>::eras_reward_points(era);
                        let treasury_account_id = Self::account_id();
                        // let (min_token_id, max_token_id) = <pallet_assets::Module<T>>::social_token_ids();

                        for (account_id, points) in reward_points.individual {
                            if let Some(controller) = <pallet_staking::Module<T>>::bonded(account_id) {
                                // let social_token_id = <pallet_social_guardians::Module<T>>::social_of(controller);
                                // if social_token_id >= min_token_id && social_token_id <= max_token_id {
                                //     // <pallet_assets::Module<T>>::issue_social_token(
                                //     //     treasury_account_id.clone(),
                                //     //     social_token_id,
                                //     //     points.into()
                                //     // );
                                //     // let _ = <pallet_assets::Module<T>>::issue(social_token_id, points.into());
                                // }
                            }
                        }

                        let next_era = era + 1;
                        NextEraForProcessing::put(next_era);
                    }
                }
            }
        }
    }
}

impl<T: Config> Module<T> {
    // Add public immutables and private mutables.

    /// The account ID of the treasury pot.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
    pub fn account_id() -> T::AccountId {
        <pallet_treasury::Module<T>>::account_id()
    }

    /// The account ID of a bounty account
    pub fn bounty_account_id(id: BountyIndex) -> T::AccountId {
        <pallet_treasury::Module<T>>::bounty_account_id(id)
    }

    /// The needed bond for a proposal whose spend is `value`.
    fn calculate_bond(value: BalanceOf<T>) -> BalanceOf<T> {
        <T as Config>::ProposalBondMinimum::get().max(<T as Config>::ProposalBond::get() * value)
    }

    /// Given a mutable reference to an `OpenTip`, insert the tip into it and check whether it
    /// closes, if so, then deposit the relevant event and set closing accordingly.
    ///
    /// `O(T)` and one storage access.
    fn insert_tip_and_check_closing(
        tip: &mut OpenTip<T::AccountId, BalanceOf<T>, T::BlockNumber, T::Hash, TokenId<T>>,
        tipper: T::AccountId,
        tip_value: BalanceOf<T>,
    ) -> bool {
        match tip.tips.binary_search_by_key(&&tipper, |x| &x.0) {
            Ok(pos) => tip.tips[pos] = (tipper, tip_value),
            Err(pos) => tip.tips.insert(pos, (tipper, tip_value)),
        }
        Self::retain_active_tips(&mut tip.tips);
        let threshold = (<T as Config>::Tippers::count() + 1) / 2;
        if tip.tips.len() >= threshold && tip.closes.is_none() {
            tip.closes =
                Some(system::Module::<T>::block_number() + <T as Config>::TipCountdown::get());
            true
        } else {
            false
        }
    }

    /// Remove any non-members of `Tippers` from a `tips` vector. `O(T)`.
    fn retain_active_tips(tips: &mut Vec<(T::AccountId, BalanceOf<T>)>) {
        let members = <T as Config>::Tippers::sorted_members();
        let mut members_iter = members.iter();
        let mut member = members_iter.next();
        tips.retain(|(ref a, _)| loop {
            match member {
                None => break false,
                Some(m) if m > a => break false,
                Some(m) => {
                    member = members_iter.next();
                    if m < a {
                        continue;
                    } else {
                        break true;
                    }
                }
            }
        });
    }

    /// Execute the payout of a tip.
    ///
    /// Up to three balance operations.
    /// Plus `O(T)` (`T` is Tippers length).
    fn payout_tip(
        hash: T::Hash,
        tip: OpenTip<T::AccountId, BalanceOf<T>, T::BlockNumber, T::Hash, TokenId<T>>,
    ) {
        let mut tips = tip.tips;
        Self::retain_active_tips(&mut tips);
        tips.sort_by_key(|i| i.1);
        let treasury = Self::account_id();
        // let max_payout = Balance::new(0); // Self::pot(tip.social_token_id);
        let mut payout = tips[tips.len() / 2].1;
        if !tip.deposit.is_zero() {
            // let _ = <pallet_assets::Module<T>>::unreserve(
            //     &tip.finder,
            //     tip.social_token_id,
            //     tip.deposit,
            // );
        }
        // if tip.finders_fee {
        //     if tip.finder != tip.who {
        //         // pay out the finder's fee.
        //         // let finders_fee = <T as Config>::TipFindersFee::get() * payout;
        //         // payout -= finders_fee;
        //         // this should go through given we checked it's at most the free balance, but still
        //         // we only make a best-effort.
        //         // let _ = <pallet_assets::Module<T>>::do_transfer(
        //         //     &treasury,
        //         //     &tip.finder,
        //         //     tip.social_token_id,
        //         //     finders_fee,
        //         //     KeepAlive,
        //         // );
        //     }
        // }
        // same as above: best-effort only.
        // let _ = <pallet_assets::Module<T>>::do_transfer(
        //     &treasury,
        //     &tip.who,
        //     tip.social_token_id,
        //     payout,
        //     KeepAlive,
        // );
        Self::deposit_event(RawEvent::TipClosed(
            hash,
            tip.who,
            tip.social_token_id,
            payout,
        ));
    }

    /// Spend some money! returns number of approvals before spend.
    fn spend_funds() -> Weight {
        let mut total_weight: Weight = Zero::zero();

        let mut budgets_remaining = vec![];
        // let (min_token_id, max_token_id) = <pallet_assets::Module<T>>::social_token_ids();
        let mut token_id: TokenId<T> = 0u32.into();
        // while token_id < min_token_id {
        //     budgets_remaining.push(0u32.into());
        //     token_id += 1u32.into();
        // }
        // while token_id <= max_token_id {
        //     // let budget_remaining = Self::pot(token_id);
        //     // budgets_remaining.push(budget_remaining);
        //     // Self::deposit_event(RawEvent::Spending(token_id, budget_remaining));
        //     token_id += 1u32.into();
        // }

        let account_id = Self::account_id();

        let mut missed_any = vec![];
        // let mut imbalances = vec![];
        token_id = 0u32.into();
        // while token_id <= max_token_id {
        //     missed_any.push(false);
        //     imbalances.push(<PositiveImbalanceOf<T>>::zero());
        //     token_id += 1u32.into();
        // }
        let proposals_len = Approvals::mutate(|v| {
            let proposals_approvals_len = v.len() as u32;
            v.retain(|&index| {
                // Should always be true, but shouldn't panic if false or we're screwed.
                if let Some(p) = Self::proposals(index) {
                    let id: usize = p.social_token_id.unique_saturated_into();
                    if p.value <= budgets_remaining[id] {
                        budgets_remaining[id] -= p.value;
                        <Proposals<T>>::remove(index);

                        // return their deposit.
                        // let _ = <pallet_assets::Module<T>>::unreserve(
                        //     &p.proposer,
                        //     p.social_token_id,
                        //     p.bond,
                        // );

                        // provide the allocation.
                        // imbalances[id].subsume(
                        //     <pallet_assets::Module<T>>::deposit_creating(
                        //         &p.beneficiary,
                        //         p.social_token_id,
                        //         p.value,
                        //     ),
                        // );

                        Self::deposit_event(RawEvent::Awarded(
                            index,
                            p.social_token_id,
                            p.value,
                            p.beneficiary,
                        ));
                        false
                    } else {
                        missed_any[id] = true;
                        true
                    }
                } else {
                    false
                }
            });
            proposals_approvals_len
        });

        total_weight += <T as Config>::WeightInfo::on_initialize_proposals(proposals_len);

        let bounties_len = BountyApprovals::mutate(|v| {
            let bounties_approval_len = v.len() as u32;
            v.retain(|&index| {
                Bounties::<T>::mutate(index, |bounty| {
                    // Should always be true, but shouldn't panic if false or we're screwed.
                    if let Some(bounty) = bounty {
                        let id: usize = bounty.social_token_id.unique_saturated_into();
                        if bounty.value <= budgets_remaining[id] {
                            budgets_remaining[id] -= bounty.value;

                            bounty.status = BountyStatus::Funded;

                            // return their deposit.
                            // let _ = <pallet_assets::Module<T>>::unreserve(
                            //     &bounty.proposer,
                            //     bounty.social_token_id,
                            //     bounty.bond,
                            // );

                            // fund the bounty account
                            // imbalances[id].subsume(
                            //     <pallet_assets::Module<T>>::deposit_creating(
                            //         &Self::bounty_account_id(index),
                            //         bounty.social_token_id,
                            //         bounty.value,
                            //     ),
                            // );

                            Self::deposit_event(RawEvent::BountyBecameActive(index));
                            false
                        } else {
                            missed_any[id] = true;
                            true
                        }
                    } else {
                        false
                    }
                })
            });
            bounties_approval_len
        });

        total_weight += <T as Config>::WeightInfo::on_initialize_bounties(bounties_len);

        // token_id = min_token_id;
        // while token_id <= max_token_id {
        //     let id: usize = token_id.unique_saturated_into();
        //     if !missed_any[id] {
        //         // burn some proportion of the remaining budget if we run a surplus.
        //         let budget_remaining: BalanceOf<T> = budgets_remaining[id];
        //         let burn = (<T as Config>::Burn::get() * budget_remaining).min(budget_remaining);
        //         budgets_remaining[id] -= burn;

        //         // let (debit, credit) = <pallet_assets::Module<T>>::pair(token_id, burn);
        //         // imbalances[id].subsume(debit);
        //         // <T as Config>::BurnDestination::on_unbalanced(credit);
        //         Self::deposit_event(RawEvent::Burnt(token_id, burn))
        //     }

        //     // Must never be an error, but better to be safe.
        //     // proof: budget_remaining is account free balance minus ED;
        //     // Thus we can't spend more than account free balance minus ED;
        //     // Thus account is kept alive; qed;
        //     // if let Err(problem) = <pallet_assets::Module<T>>::settle(
        //     //     &account_id,
        //     //     token_id,
        //     //     imbalances[id].clone(),
        //     //     WithdrawReasons::TRANSFER,
        //     //     KeepAlive,
        //     // ) {
        //     //     print("Inconsistent state - couldn't settle imbalance for funds spent by treasury");
        //     //     // Nothing else to do here.
        //     //     drop(problem);
        //     // }

        //     Self::deposit_event(RawEvent::Rollover(token_id, budgets_remaining[id]));

        //     token_id += 1u32.into();
        // }

        total_weight
    }

    /// Return the amount of money in the pot.
    // The existential deposit is not part of the pot so treasury account never gets deleted.
    // fn pot(token_id: TokenId<T>) -> BalanceOf<T> {
    //     <pallet_assets::Module<T>>::free_balance(&Self::account_id(), token_id)
    //         .saturating_sub(<pallet_assets::Module<T>>::minimum_balance())
    // }

    fn create_bounty(
        proposer: T::AccountId,
        description: Vec<u8>,
        token_id: TokenId<T>,
        value: BalanceOf<T>,
    ) -> DispatchResult {
        ensure!(
            description.len() <= <T as Config>::MaximumReasonLength::get() as usize,
            Error::<T>::ReasonTooBig
        );
        ensure!(
            value >= <T as Config>::BountyValueMinimum::get(),
            Error::<T>::InvalidValue
        );

        let index = Self::bounty_count();

        // reserve deposit for new bounty
        let bond = <T as Config>::BountyDepositBase::get()
            + <T as Config>::DataDepositPerByte::get() * (description.len() as u32).into();
        // <pallet_assets::Module<T>>::reserve(&proposer, token_id, bond)
        //     .map_err(|_| Error::<T>::InsufficientProposersBalance)?;

        BountyCount::put(index + 1);

        let bounty = Bounty {
            proposer,
            value,
            fee: 0u32.into(),
            curator_deposit: 0u32.into(),
            bond,
            status: BountyStatus::Proposed,
            social_token_id: token_id,
        };

        Bounties::<T>::insert(index, &bounty);
        BountyDescriptions::insert(index, description);

        Self::deposit_event(RawEvent::BountyProposed(index));

        Ok(())
    }
}
