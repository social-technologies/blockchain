#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::Get,
};
use frame_system::ensure_signed;
use pallet_staking::EraIndex;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Config: frame_system::Config + pallet_assets::Config + pallet_staking::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
    trait Store for Module<T: Config> as ValidatorRegistry {
        /// Map from the controller account to the social token id.
        ChampionOf get(fn champion_of): map hasher(blake2_128_concat) T::AccountId => T::AssetId;
        /// Map from the social token id to the vector of controller accounts.
        ChampionsOfSocialToken get(fn champions_of_social_token): map hasher(blake2_128_concat) T::AssetId => Vec<T::AccountId>;
        /// Current champions (controller accounts).
        Champions get(fn champions): Vec<T::AccountId>;
        /// Map from the era index to the vector of controller accounts.
        ChampionHistory get(fn champion_history): map hasher(blake2_128_concat) EraIndex => Vec<T::AccountId>;
        /// Map from (era index, controller account) to the social token id.
        pub ChampionDetailHistory get(fn champion_detail_history): double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId => T::AssetId;
        /// Number of eras to keep in history.
        HistoryDepth get(fn history_depth): u32 = 84;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        AssetId = <T as pallet_assets::Config>::AssetId,
    {
        Registered(AccountId, AssetId),
        Unregistered(AccountId, AssetId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        AlreadyRegistered,
        NotFound,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn register(origin, social_token_id: T::AssetId) -> dispatch::DispatchResult {
            let validator = ensure_signed(origin)?;

            <pallet_assets::Module<T>>::validate_asset_id(social_token_id)?;
            ensure!(!<ChampionOf<T>>::contains_key(&validator), Error::<T>::AlreadyRegistered);

            <ChampionOf<T>>::insert(&validator, social_token_id);
            <ChampionsOfSocialToken<T>>::mutate(social_token_id, |validators| {
                validators.push(validator.clone())
            });
            <Champions<T>>::mutate(|validators| {
                validators.push(validator.clone())
            });

            Self::deposit_event(RawEvent::Registered(validator, social_token_id));
            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn unregister(origin) -> dispatch::DispatchResult {
            let validator = ensure_signed(origin)?;
            let social_token_id = <ChampionOf<T>>::try_get(&validator)
                .map_err(|_| Error::<T>::NotFound)?;

            <ChampionOf<T>>::remove(&validator);
            <ChampionsOfSocialToken<T>>::mutate(social_token_id, |validators| {
                validators.retain(|account_id| account_id != &validator)
            });
            <Champions<T>>::mutate(|validators| {
                validators.retain(|account_id| account_id != &validator)
            });

            Self::deposit_event(RawEvent::Unregistered(validator, social_token_id));
            Ok(())
        }

        fn on_finalize() {
            let current_era = pallet_staking::CurrentEra::get().unwrap_or(0);
            Self::clean_history(current_era);
            Self::update_history(current_era);
        }

    }
}

impl<T: Config> Module<T> {
    fn clean_history(current_era: EraIndex) {
        let history_depth = HistoryDepth::get();
        match current_era.checked_sub(history_depth) {
            Some(era) => {
                <ChampionHistory<T>>::remove(era);
                <ChampionDetailHistory<T>>::remove_prefix(era);
            }
            None => (),
        }
    }

    fn update_history(current_era: EraIndex) {
        let champions = <Champions<T>>::get();
        champions.iter().for_each(|champion| {
            let social_token_id = <ChampionOf<T>>::get(champion);
            <ChampionDetailHistory<T>>::insert(current_era, champion, social_token_id)
        });
        <ChampionHistory<T>>::insert(current_era, champions);
    }
}
