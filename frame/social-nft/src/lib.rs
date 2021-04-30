// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::Get,
};
use frame_system::{self as system, ensure_signed};
use sp_core::U256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use sp_runtime::traits::{Zero, One};
use sp_runtime::{
	traits::Saturating,
};
mod mock;
mod tests;

type NftId = U256;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Erc721Token<T: Config> {
    pub id: NftId,
    pub metadata: Vec<u8>,
	pub royalty: T::Balance,
}

pub trait Config: system::Config + pallet_assets::Config + pallet_timestamp::Config {
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;
    /// Some identifier for this token type, possibly the originating ethereum address.
    /// This is not explicitly used for anything, but may reflect the bridge's notion of resource ID.
    type Identifier: Get<[u8; 32]>;
}

decl_event! {
    pub enum Event<T>
    where
        <T as system::Config>::AccountId,
    {
        /// New token created
        Minted(AccountId, NftId),
        /// Token transfer between two parties
        Transferred(AccountId, AccountId, NftId),
        /// Token removed from the system
        Burned(NftId),
		/// Set Ask Amount
		SetAskAmount(NftId),
		/// Set Ask Amount
		SetBidAmount(NftId),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// ID not recognized
        NftIdDoesNotExist,
        /// Already exists with an owner
        TokenAlreadyExists,
        /// Origin is not owner
        NotOwner,
		/// Not For Sale
        NotForSale,
    }
}

decl_storage! {
    trait Store for Module<T: Config> as SocialNFT {
        /// Maps tokenId to Erc721 object
        pub Tokens get(fn tokens): map hasher(opaque_blake2_256) NftId => Option<Erc721Token<T>>;
        /// Maps tokenId to owner
        pub TokenCreatorOwner get(fn owner_of): map hasher(opaque_blake2_256) NftId => (T::AccountId, T::AccountId);
        /// Total number of tokens in existence
        pub TokenCount get(fn token_count): U256 = U256::zero();
        /// Maximum token id
        pub MaxTokenId get(fn max_token_id): U256 = U256::zero();
		/// Set ask amount for token id
		pub TokenAskAmount get(fn ask_token): double_map hasher(opaque_blake2_256) NftId,
		hasher(twox_64_concat) T::AssetId => (T::Balance, T::AccountId);
		 /// Maps tokenId to owner
        pub TokenBidAmount get(fn bid_token): map hasher(opaque_blake2_256) T::AccountId =>
		(T::Balance, T::AssetId, T::Moment);
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        /// Creates a new token with the given token ID and metadata, and gives ownership to owner
        #[weight = 195_000_000]
        pub fn mint(origin, owner: T::AccountId, id: NftId, metadata: Vec<u8>, royalty: T::Balance) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            Self::mint_token(owner, id, metadata, royalty)?;

            Ok(())
        }

        /// Changes ownership of a token sender owns
        #[weight = 195_000_000]
        pub fn transfer(origin, to: T::AccountId, id: NftId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::transfer_from(sender, to, id)?;

            Ok(())
        }

        /// Remove token from the system
        #[weight = 195_000_000]
        pub fn burn(origin, id: NftId) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            ensure!(TokenCreatorOwner::<T>::contains_key(id), Error::<T>::NftIdDoesNotExist);
        	let (_, owner) = Self::owner_of(id);

            Self::burn_token(owner, id)?;

            Ok(())
        }

		#[weight = 195_000_000]
        pub fn set_ask(origin, nft_id: NftId, token_id: T::AssetId, amount: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(TokenCreatorOwner::<T>::contains_key(nft_id), Error::<T>::NftIdDoesNotExist);
        	let (_, owner) = Self::owner_of(nft_id);
			ensure!(owner == sender, Error::<T>::NotOwner);
            Self::set_ask_token(owner, nft_id, token_id, amount)?;

            Ok(())
        }

		#[weight = 195_000_000]
        pub fn set_bid(origin, nft_id: NftId, token_id: T::AssetId, amount: T::Balance, dead_line: T::Moment) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(TokenCreatorOwner::<T>::contains_key(nft_id), Error::<T>::NftIdDoesNotExist);

            Self::set_bid_token(sender, nft_id, token_id, amount, dead_line)?;

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    /// Creates a new token in the system.
    pub fn mint_token(owner: T::AccountId, id: NftId, metadata: Vec<u8>, royalty: T::Balance) -> DispatchResult {
        ensure!(!<Tokens<T>>::contains_key(id), Error::<T>::TokenAlreadyExists);

        let new_token = Erc721Token { id, metadata, royalty };

		<Tokens<T>>::insert(&id, new_token);
        <TokenCreatorOwner<T>>::insert(&id, (owner.clone(), owner.clone()));
        let new_total = <TokenCount>::get().saturating_add(U256::one());
        <TokenCount>::put(new_total);
        if <MaxTokenId>::get() < id {
            <MaxTokenId>::put(id)
        }

        Self::deposit_event(RawEvent::Minted(owner, id));

        Ok(())
    }

    /// Modifies ownership of a token
    pub fn transfer_from(from: T::AccountId, to: T::AccountId, id: NftId) -> DispatchResult {
        // Check from is owner and token exists
		ensure!(TokenCreatorOwner::<T>::contains_key(id), Error::<T>::NftIdDoesNotExist);
        let (_, owner) = Self::owner_of(id);
        ensure!(owner == from, Error::<T>::NotOwner);
        // Update owner
		TokenCreatorOwner::<T>::mutate(id, |(_, owner)| *owner = to.clone());

        Self::deposit_event(RawEvent::Transferred(from, to, id));

        Ok(())
    }

    /// Deletes a token from the system.
    pub fn burn_token(from: T::AccountId, id: NftId) -> DispatchResult {
		ensure!(TokenCreatorOwner::<T>::contains_key(id), Error::<T>::NftIdDoesNotExist);
		let (_, owner) = Self::owner_of(id);
        ensure!(owner == from, Error::<T>::NotOwner);

		<Tokens<T>>::remove(&id);
        <TokenCreatorOwner<T>>::remove(&id);
        let new_total = <TokenCount>::get().saturating_sub(U256::one());
        <TokenCount>::put(new_total);

        Self::deposit_event(RawEvent::Burned(id));

        Ok(())
    }

	pub fn set_ask_token(owner: T::AccountId, id: NftId, token_id: T::AssetId, amount: T::Balance) -> DispatchResult {

		<TokenAskAmount<T>>::insert(&id, token_id, (amount, owner));
		Self::deposit_event(RawEvent::SetAskAmount(id));
		Ok(())
	}

	pub fn set_bid_token(sender: T::AccountId, id: NftId, token_id: T::AssetId, amount: T::Balance, dead_line: T::Moment) -> DispatchResult {

		let (ask_token, _) = TokenAskAmount::<T>::get(id, token_id);

		ensure!(ask_token.is_zero(), Error::<T>::NotForSale);
		let now_timestamp = <pallet_timestamp::Module<T>>::now();

		if amount > ask_token && dead_line >=now_timestamp {
			Self::execute_trade(sender, id, token_id, amount);
		} else {
			<TokenBidAmount<T>>::insert(&sender, (amount, token_id, dead_line));
			Self::deposit_event(RawEvent::SetBidAmount(id));
		}

		Ok(())
	}

	fn execute_trade(sender: T::AccountId, id: NftId, token_id: T::AssetId, amount: T::Balance) -> DispatchResult {
		let nft = Tokens::<T>::get(id).ok_or(Error::<T>::NftIdDoesNotExist)?;
		let (creator, owner) = Self::owner_of(id);
		if nft.royalty.is_zero() {
			<pallet_assets::Module<T>>::do_transfer(token_id, sender.clone(), creator, amount.saturating_mul(nft.royalty));
			<pallet_assets::Module<T>>::do_transfer(token_id, sender, owner, amount.saturating_mul(T::Balance::one().saturating_sub(nft.royalty)));
			TokenCreatorOwner::<T>::mutate(id, |(_, owner)| *owner = sender.clone());
		}
		Ok(())
	}
}
