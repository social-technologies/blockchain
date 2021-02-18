
#![cfg_attr(not(feature = "std"), no_std)]
/// TODO: add events
/// set NextAssetId's initial value to 1
use frame_support::{
	Parameter, decl_module, decl_event, decl_storage, decl_error, ensure,
};
use sp_runtime::{
	RuntimeDebug, DispatchResult, DispatchError,
	traits::{
		CheckedSub, Saturating, Member, AtLeast32Bit, AtLeast32BitUnsigned, Zero, StaticLookup
	},
};
use frame_system::ensure_signed;
use sp_runtime::traits::One;
use codec::{Encode, Decode};

type Symbol = [u8; 8];
const UNI_V1: Symbol = *b"UNISWAP1";

/// TODO: consider if there need to be more fields
#[derive(Encode, Decode)]
pub struct TokenDossier {
	pub symbol: Symbol
}

impl TokenDossier {
	pub fn new_lp_token() -> Self {
		TokenDossier {
			symbol: UNI_V1
		}
	}
}

/// The module configuration trait.
pub trait Trait: frame_system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The units in which we record balances.
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

	/// The arithmetic type of asset identifier.
	type AssetId: Parameter + AtLeast32Bit + Default + Copy;

    type ExchangeId: Parameter + Member + AtLeast32Bit + Default + Copy;
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
		<T as Trait>::AssetId,
	{
		/// Some assets were issued. [asset_id, owner, total_supply]
		Issued(AssetId, AccountId, Balance),
		/// Some assets were transferred. [asset_id, from, to, amount]
		Transferred(AssetId, AccountId, AccountId, Balance),
		/// Some assets were destroyed. [asset_id, owner, balance]
		Destroyed(AssetId, AccountId, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Transfer amount should be non-zero
		AmountZero,
		/// Account balance must be greater than or equal to the transfer amount
		BalanceLow,
		/// Balance should be non-zero
		BalanceZero,
		/// Have no permission to transfer someone's balance
		NotAllowed,
		/// Asset has not been created
		AssetNotExists,
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as FungibleAssets {
		TokenInfo get(fn token_info): map hasher(twox_64_concat) T::AssetId => Option<TokenDossier>;
		/// The number of units of assets held by any given account.
		Balances get(fn balances):
			double_map hasher(twox_64_concat) T::AssetId, hasher(blake2_128_concat) T::AccountId => T::Balance;
		/// Allowance
		Allowances get(fn allowances):
			double_map  hasher(twox_64_concat) T::AssetId, hasher(blake2_128_concat) (T::AccountId, T::AccountId) => T::Balance;
		/// The next asset identifier up for grabs.
		NextAssetId get(fn next_asset_id): T::AssetId;
		/// The total unit supply of an asset.
		///
		/// TWOX-NOTE: `AssetId` is trusted, so this is safe.
		TotalSupply get(fn total_supply): map hasher(twox_64_concat) T::AssetId => T::Balance;
	}
}


decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Move some assets from one holder to another.
		///
		/// # <weight>
		/// - `O(1)`
		/// - 1 static lookup
		/// - 2 storage mutations (codec `O(1)`).
		/// - 1 event.
		/// # </weight>
		#[weight = 0]
		fn transfer(origin,
			#[compact] id: T::AssetId,
			target: T::AccountId,
			#[compact] amount: T::Balance
		) {
			let origin = ensure_signed(origin)?;

			<Self as Fungible<_, _>>::transfer(&id, &origin, &target, amount)?;
			Self::deposit_event(RawEvent::Transferred(id, origin, target.clone(), amount));
		}
	}
}


// The main implementation block for the module.
impl<T: Trait> Module<T> {

	pub fn impl_transfer(asset_id: &T::AssetId, from: &T::AccountId, to: &T::AccountId, value: T::Balance) -> DispatchResult {
		let new_balance = Self::balances(asset_id, from)
			.checked_sub(&value)
			.ok_or(Error::<T>::BalanceLow)?;

		if from != to {
			<Balances<T>>::mutate(asset_id, from, |balance| *balance -= value);
			<Balances<T>>::mutate(asset_id, to, |balance| *balance += value);
		}

		Ok(())
	}
}


pub trait Fungible<AssetId, AccountId> {
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

	fn total_supply(asset_id: &AssetId) -> Self::Balance;
	fn balances(asset_id: &AssetId, who: &AccountId) -> Self::Balance;
	fn allowances(asset_id: &AssetId, owner: &AccountId, spender: &AccountId) -> Self::Balance;
	fn transfer(asset_id: &AssetId, from: &AccountId, to: &AccountId, value: Self::Balance) -> DispatchResult;
	fn transfer_from(asset_id: &AssetId, from: &AccountId, operator: &AccountId, to: &AccountId, value: Self::Balance) -> DispatchResult;
}

pub trait IssueAndBurn<AssetId, AccountId>: Fungible<AssetId, AccountId> {

	fn exists(asset_id: &AssetId) -> bool;

	fn create_new_asset(dossier: TokenDossier) -> AssetId;

	fn issue(asset_id: &AssetId, who: &AccountId, value: Self::Balance) -> DispatchResult;

	fn burn(asset_id: &AssetId, who: &AccountId, value: Self::Balance) -> DispatchResult;
}


impl<T: Trait> Fungible<T::AssetId, T::AccountId> for Module<T> {
	type Balance = T::Balance;

	fn total_supply(asset_id: &T::AssetId) -> Self::Balance {
		Self::total_supply(&asset_id)
	}

	fn balances(asset_id: &T::AssetId, who: &T::AccountId) -> Self::Balance {
		Self::balances(asset_id, who)
	}

	fn allowances(asset_id: &T::AssetId, owner: &T::AccountId, spender: &T::AccountId) -> Self::Balance {
		Self::allowances(asset_id, (owner, spender))
	}

	fn transfer(asset_id: &T::AssetId, from: &T::AccountId, to: &T::AccountId, value: Self::Balance) -> DispatchResult {
		Self::impl_transfer(asset_id, from, to, value)
	}

	fn transfer_from(asset_id: &T::AssetId, from: &T::AccountId, operator: &T::AccountId, to: &T::AccountId, value: Self::Balance) -> DispatchResult {

		let new_allowance = Self::allowances(asset_id, (from, operator))
			.checked_sub(&value)
			.ok_or(Error::<T>::NotAllowed)?;

		if from != to {
			Self::impl_transfer(asset_id, from, to, value)?;
		}

		<Allowances<T>>::mutate(asset_id, (from, operator), |approved_balance| {
			*approved_balance = new_allowance;
		});

		Ok(())
	}
}

impl<T: Trait> IssueAndBurn<T::AssetId, T::AccountId> for Module<T> {


	fn exists(asset_id: &T::AssetId) -> bool {
		Self::token_info(asset_id).is_some()
	}

	fn create_new_asset(dossier: TokenDossier) -> T::AssetId {
		let id = Self::next_asset_id();
		<NextAssetId<T>>::mutate(|id| *id += One::one());

		<TokenInfo<T>>::insert(id, dossier);
		id
	}

	fn issue(asset_id: &T::AssetId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		ensure!(Self::exists(asset_id), Error::<T>::AssetNotExists);

		<Balances<T>>::mutate(asset_id, who, |balance| {
			*balance = balance.saturating_add(value);
		});
		<TotalSupply<T>>::mutate(asset_id, |supply| {
			*supply = supply.saturating_add(value);
		});

		Self::deposit_event(RawEvent::Issued(asset_id.clone(), who.clone(), value));

		Ok(())
	}

	fn burn(asset_id: &T::AssetId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		ensure!(Self::exists(asset_id), Error::<T>::AssetNotExists);
		let new_balance = Self::balances(asset_id, who)
			.checked_sub(&value)
			.ok_or(Error::<T>::BalanceLow)?;

		<Balances<T>>::mutate(asset_id, who, |balance| *balance = new_balance);
		<TotalSupply<T>>::mutate(asset_id, |supply| {
			*supply = supply.saturating_sub(value);
		});

		Ok(())
	}
}
