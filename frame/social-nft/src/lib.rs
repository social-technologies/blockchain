// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::Get,
};
use frame_system::{self as system, ensure_signed};
use sp_core::U256;
use sp_runtime::{
    RuntimeDebug,
    traits::StaticLookup,
};
use sp_std::prelude::*;

mod mock;
mod tests;

type NftId = U256;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Erc721Token {
    pub id: NftId,
    pub metadata: Vec<u8>,
}

pub trait Config: system::Config {
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
        /// New token created. \[owner, token_id\]
        Minted(AccountId, NftId),
        /// Token transfer between two parties. \[from, to. token_id\]
        Transferred(AccountId, AccountId, NftId),
        /// Token removed from the system. \[token_id\]
        Burned(NftId),
        /// Approval for all. \[owner, to, approved\]
        ApprovalForAll(AccountId, AccountId, bool),
        /// Token Approval to an account. \[sender, to, token_id\]
        Approval(AccountId, AccountId, NftId),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// ID not recognized.
        NftIdDoesNotExist,
        /// Already exists with an owner.
        TokenAlreadyExists,
        /// Origin is not the owner.
        NotOwner,
        /// Origin is not the owner or approved for all.
        NotOwnerOrApprovedForAll,
        /// Attempt to approval to the current owner.
        ApprovalToCurrentOwner,
        /// Attempt to approve to caller.
        ApproveToCaller,
    }
}

decl_storage! {
    trait Store for Module<T: Config> as SocialNFT {
        /// Maps token ID to Erc721 object.
        pub Tokens get(fn tokens): map hasher(opaque_blake2_256) NftId => Option<Erc721Token>;
        /// Maps token ID to owner.
        pub OwnerOf get(fn owner_of): map hasher(opaque_blake2_256) NftId => Option<T::AccountId>;
        /// Total number of tokens in existence.
        pub TokenCount get(fn token_count): U256 = U256::zero();
        /// Maximum token ID.
        pub MaxTokenId get(fn max_token_id): U256 = U256::zero();
        /// Query if an address is an authorized operator for another address.
        /// The first account ID is an owner's address, the second account ID is an operator's address.
        pub IsApprovedForAll get(fn is_approved_for_all):
            double_map hasher(twox_64_concat) T::AccountId, hasher(twox_64_concat) T::AccountId => bool = false;
        /// Mapping from token ID to approved address.
        pub TokenApprovals get(fn token_approvals): map hasher(opaque_blake2_256) NftId => T::AccountId;
        /// The total number of tokens what has an account.
        pub BalanceOf get(fn balance_of): map hasher(opaque_blake2_256) T::AccountId => U256 = U256::zero();
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        /// Creates a new token with the given token ID and metadata, and gives ownership to owner
        #[weight = 195_000_000]
        pub fn mint(origin, owner: <T::Lookup as StaticLookup>::Source, id: NftId, metadata: Vec<u8>) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            let owner = T::Lookup::lookup(owner)?;

            Self::do_mint(owner, id, metadata)?;

            Ok(())
        }

        /// Changes ownership of a token sender owns
        #[weight = 195_000_000]
        pub fn transfer(origin, to: <T::Lookup as StaticLookup>::Source, id: NftId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;

            Self::do_transfer(sender, to, id)?;

            Ok(())
        }

        /// Remove token from the system
        #[weight = 195_000_000]
        pub fn burn(origin, id: NftId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::do_burn(sender, id)?;

            Ok(())
        }
    
        /// Approve token to another address.
        #[weight = 195_000_000]
        pub fn approve(origin, to: <T::Lookup as StaticLookup>::Source, id: NftId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;

            Self::do_approve(sender, to, id)?;

            Ok(())
        }

        /// Set approval for another address.
        #[weight = 195_000_000]
        pub fn set_approval_for_all(
            origin,
            operator: <T::Lookup as StaticLookup>::Source,
            approved: bool
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let operator = T::Lookup::lookup(operator)?;

            Self::do_set_approval_for_all(sender, operator, approved)?;

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    /// Creates a new token in the system.
    pub fn do_mint(owner: T::AccountId, id: NftId, metadata: Vec<u8>) -> DispatchResult {
        ensure!(!Tokens::contains_key(id), Error::<T>::TokenAlreadyExists);

        let new_token = Erc721Token { id, metadata };

        <Tokens>::insert(&id, new_token);
        <OwnerOf<T>>::insert(&id, &owner);
        let new_balance = <BalanceOf<T>>::get(&owner).saturating_add(U256::one());
        <BalanceOf<T>>::insert(&owner, new_balance);
        let new_total = <TokenCount>::get().saturating_add(U256::one());
        <TokenCount>::put(new_total);
        if <MaxTokenId>::get() < id {
            <MaxTokenId>::put(id)
        }

        Self::deposit_event(RawEvent::Minted(owner, id));

        Ok(())
    }

    /// Modifies ownership of a token
    pub fn do_transfer(from: T::AccountId, to: T::AccountId, id: NftId) -> DispatchResult {
        // Check from is owner and token exists
        let owner = Self::owner_of(id).ok_or(Error::<T>::NftIdDoesNotExist)?;
        ensure!(owner == from, Error::<T>::NotOwner);
        // Update owner
        <OwnerOf<T>>::insert(&id, to.clone());
        // Clear approvals from the previous owner
        <TokenApprovals<T>>::remove(id);

        Self::deposit_event(RawEvent::Transferred(from, to, id));

        Ok(())
    }

    /// Deletes a token from the system.
    pub fn do_burn(from: T::AccountId, id: NftId) -> DispatchResult {
        let owner = Self::owner_of(id).ok_or(Error::<T>::NftIdDoesNotExist)?;
        ensure!(owner == from, Error::<T>::NotOwner);

        <Tokens>::remove(&id);
        <OwnerOf<T>>::remove(&id);
        // Clear approvals from the previous owner
        <TokenApprovals<T>>::remove(id);
        let new_balance = <BalanceOf<T>>::get(&owner).saturating_sub(U256::one());
        <BalanceOf<T>>::insert(&owner, new_balance);
        let new_total = <TokenCount>::get().saturating_sub(U256::one());
        <TokenCount>::put(new_total);

        Self::deposit_event(RawEvent::Burned(id));

        Ok(())
    }

    pub fn do_approve(from: T::AccountId, to: T::AccountId, id: NftId) -> DispatchResult {
        let owner = Self::owner_of(id).ok_or(Error::<T>::NftIdDoesNotExist)?;
        ensure!(
            &owner == &from || Self::is_approved_for_all(&owner, &from),
            Error::<T>::NotOwnerOrApprovedForAll
        );
        ensure!(owner != to, Error::<T>::ApprovalToCurrentOwner);
        <TokenApprovals<T>>::insert(id, &to);

        Self::deposit_event(RawEvent::Approval(owner, to, id));

        Ok(())
    }

    pub fn do_set_approval_for_all(
        sender: T::AccountId,
        operator: T::AccountId,
        approved: bool
    ) -> DispatchResult {
        ensure!(sender != operator, Error::<T>::ApproveToCaller);

        <IsApprovedForAll<T>>::insert(&sender, &operator, approved);

        Self::deposit_event(RawEvent::ApprovalForAll(sender, operator, approved));

        Ok(())
    }
}
