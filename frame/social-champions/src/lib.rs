#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::Get,
};
use frame_system::ensure_signed;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Config: frame_system::Config + pallet_social_tokens::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
    trait Store for Module<T: Config> as ValidatorRegistry {
        ChampionOf get(fn social_of): map hasher(blake2_128_concat) T::AccountId => T::SocialTokenId;
        Validators get(fn validators): map hasher(blake2_128_concat) T::SocialTokenId => Vec<T::AccountId>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        SocialTokenId = <T as pallet_social_tokens::Config>::SocialTokenId,
    {
        Registered(AccountId, SocialTokenId),
        Unregistered(AccountId, SocialTokenId),
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
        pub fn register(origin, social_token_id: T::SocialTokenId) -> dispatch::DispatchResult {
            let validator = ensure_signed(origin)?;

            <pallet_social_tokens::Module<T>>::validate_social_token_id(social_token_id)?;
            ensure!(!<ChampionOf<T>>::contains_key(&validator), Error::<T>::AlreadyRegistered);

            <ChampionOf<T>>::insert(&validator, social_token_id);
            <Validators<T>>::mutate(social_token_id, |validators| {
                validators.push(validator.clone())
            });

            Self::deposit_event(RawEvent::Registered(validator, social_token_id));
            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn unregister(origin) -> dispatch::DispatchResult {
            let validator = ensure_signed(origin)?;

            ensure!(<ChampionOf<T>>::contains_key(&validator), Error::<T>::NotFound);

            let social_token_id = <ChampionOf<T>>::get(&validator);
            <ChampionOf<T>>::remove(&validator);
            <Validators<T>>::mutate(social_token_id, |validators| {
                validators.retain(|account_id| account_id != &validator)
            });

            Self::deposit_event(RawEvent::Unregistered(validator, social_token_id));
            Ok(())
        }
    }
}
