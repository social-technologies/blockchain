#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Get;
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure};
use frame_system::ensure_signed;
use pallet_assets::IssueAndBurn;
use pallet_staking::{EraIndex, WeightInfo as StakingWeightInfo};
use sp_runtime::traits::SaturatedConversion;
use sp_runtime::{traits::Zero, DispatchResult, Perbill};
use sp_std::prelude::*;
pub use weights::WeightInfo;

mod default_weights;
pub mod weights;

type BalanceOf<T> = <T as pallet_assets::Config>::Balance;

pub trait Config:
    frame_system::Config
    + pallet_staking::Config
    + pallet_assets::Config
    + pallet_social_guardians::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    type FungibleToken: IssueAndBurn<Self::AssetId, Self::AccountId>;

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
}

decl_storage! {
    trait Store for Module<T: Config> as SocialTreasury {
        /// List of eras for which the stakers behind a validator have claimed rewards. Only updated
        /// for validators.
        pub ClaimedRewards get(fn claimed_rewards): map hasher(blake2_128_concat) T::AccountId => Vec<EraIndex>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        AssetId = <T as pallet_assets::Config>::AssetId,
        Balance = BalanceOf<T>,
    {
        /// The staker has been rewarded by this amount. \[asset_id, stash, amount\]
        Reward(AssetId, AccountId, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The call is not allowed at the given time due to restrictions of election period.
        CallNotAllowed,
        /// Not a controller account.
        NotController,
        /// Not a stash account.
        NotStash,
        /// Invalid era to reward.
        InvalidEraToReward,
        /// Rewards for this era have already been claimed for this validator.
        AlreadyClaimed,
        IsNotGuardian,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Pay out all the stakers behind a single validator for a single era.
        ///
        /// - `validator_stash` is the stash account of the validator. Their nominators, up to
        ///   `T::MaxNominatorRewardedPerValidator`, will also receive their rewards.
        /// - `era` may be any era between `[current_era - history_depth; current_era]`.
        ///
        /// The origin of this call must be _Signed_. Any account can call this function, even if
        /// it is not one of the stakers.
        ///
        /// This can only be called when [`EraElectionStatus`] is `Closed`.
        ///
        /// # <weight>
        /// - Time complexity: at most O(MaxNominatorRewardedPerValidator).
        /// - Contains a limited number of reads and writes.
        /// -----------
        /// N is the Number of payouts for the validator (including the validator)
        /// Weight:
        /// - Reward Destination Staked: O(N)
        /// - Reward Destination Controller (Creating): O(N)
        /// DB Weight:
        /// - Read: EraElectionStatus, CurrentEra, HistoryDepth, ErasValidatorReward,
        ///         ErasStakersClipped, ErasRewardPoints, ErasValidatorPrefs (8 items)
        /// - Read Each: Bonded, Ledger, Payee, Locks, System Account (5 items)
        /// - Write Each: System Account, Locks, Ledger (3 items)
        ///
        ///   NOTE: weights are assuming that payouts are made to alive stash account (Staked).
        ///   Paying even a dead controller is cheaper weight-wise. We don't do any refunds here.
        /// # </weight>
        #[weight = <T as pallet_staking::Config>::WeightInfo::payout_stakers_alive_staked(<T as pallet_staking::Config>::MaxNominatorRewardedPerValidator::get())]
        fn payout_stakers(origin, validator_stash: T::AccountId, era: EraIndex) -> DispatchResult {
            ensure!(<pallet_staking::Module<T>>::era_election_status().is_closed(), Error::<T>::CallNotAllowed);
            ensure_signed(origin)?;
            Self::do_payout_stakers(validator_stash, era)
        }
    }
}

impl<T: Config> Module<T> {
    fn do_payout_stakers(validator_stash: T::AccountId, era: EraIndex) -> DispatchResult {
        // Validate input data
        let current_era =
            pallet_staking::CurrentEra::get().ok_or(Error::<T>::InvalidEraToReward)?;
        ensure!(era <= current_era, Error::<T>::InvalidEraToReward);
        let history_depth = <pallet_staking::Module<T>>::history_depth();
        let guardians_history_depth = <pallet_social_guardians::Module<T>>::history_depth();
        ensure!(
            era >= current_era.saturating_sub(history_depth)
                && era >= current_era.saturating_sub(guardians_history_depth),
            Error::<T>::InvalidEraToReward
        );

        // Note: if era has no reward to be claimed, era may be future. better not to update
        // `ClaimedRewards` in this case.
        let era_payout = <pallet_staking::ErasValidatorReward<T>>::get(&era)
            .ok_or_else(|| Error::<T>::InvalidEraToReward)?;

        let controller =
            <pallet_staking::Module<T>>::bonded(&validator_stash).ok_or(Error::<T>::NotStash)?;
        let ledger = <pallet_staking::Ledger<T>>::get(&controller)
            .ok_or_else(|| Error::<T>::NotController)?;
        let asset_id =
            <pallet_social_guardians::GuardianDetailHistory<T>>::try_get(&era, &controller)
                .map_err(|_| Error::<T>::IsNotGuardian)?;

        let mut claimed_rewards = Self::claimed_rewards(&controller);
        match claimed_rewards.binary_search(&era) {
            Ok(_) => Err(Error::<T>::AlreadyClaimed)?,
            Err(pos) => claimed_rewards.insert(pos, era),
        }

        let exposure = <pallet_staking::ErasStakersClipped<T>>::get(&era, &ledger.stash);

        /* Input data seems good, no errors allowed after this point */

        <ClaimedRewards<T>>::insert(&controller, &claimed_rewards);

        // Get Era reward points. It has TOTAL and INDIVIDUAL
        // Find the fraction of the era reward that belongs to the validator
        // Take that fraction of the eras rewards to split to nominator and validator
        //
        // Then look at the validator, figure out the proportion of their reward
        // which goes to them and each of their nominators.

        let era_reward_points = <pallet_staking::ErasRewardPoints<T>>::get(&era);
        let total_reward_points = era_reward_points.total;
        let validator_reward_points = era_reward_points
            .individual
            .get(&ledger.stash)
            .map(|points| *points)
            .unwrap_or_else(|| Zero::zero());

        // Nothing to do if they have no reward points.
        if validator_reward_points.is_zero() {
            return Ok(());
        }

        // This is the fraction of the total reward that the validator and the
        // nominators will get.
        let validator_total_reward_part =
            Perbill::from_rational_approximation(validator_reward_points, total_reward_points);

        // This is how much validator + nominators are entitled to.
        let validator_total_payout = validator_total_reward_part * era_payout;

        let validator_prefs = <pallet_staking::ErasValidatorPrefs<T>>::get(&era, &validator_stash);
        // Validator first gets a cut off the top.
        let validator_commission = validator_prefs.commission;
        let validator_commission_payout = validator_commission * validator_total_payout;

        let validator_leftover_payout = validator_total_payout - validator_commission_payout;
        // Now let's calculate how this is split to the validator.
        let validator_exposure_part =
            Perbill::from_rational_approximation(exposure.own, exposure.total);
        let validator_staking_payout = validator_exposure_part * validator_leftover_payout;

        // We can now make total validator payout:
        let total_validator_payout =
            (validator_staking_payout + validator_commission_payout).saturated_into::<u128>();
        if let Ok(()) = T::FungibleToken::issue(
            &asset_id,
            &ledger.stash,
            total_validator_payout.saturated_into(),
        ) {
            Self::deposit_event(RawEvent::Reward(
                asset_id,
                ledger.stash,
                total_validator_payout.saturated_into(),
            ));
        }

        // Lets now calculate how this is split to the nominators.
        // Reward only the clipped exposures. Note this is not necessarily sorted.
        for nominator in exposure.others.iter() {
            let nominator_exposure_part =
                Perbill::from_rational_approximation(nominator.value, exposure.total);

            let nominator_reward =
                (nominator_exposure_part * validator_leftover_payout).saturated_into::<u128>();
            // We can now make nominator payout:
            if let Ok(()) = T::FungibleToken::issue(
                &asset_id,
                &nominator.who,
                nominator_reward.saturated_into(),
            ) {
                Self::deposit_event(RawEvent::Reward(
                    asset_id,
                    nominator.who.clone(),
                    nominator_reward.saturated_into(),
                ));
            }
        }

        Ok(())
    }
}
