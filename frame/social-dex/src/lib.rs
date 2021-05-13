#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Currency, ExistenceRequirement, Get},
};
use frame_system::{ensure_root, ensure_signed};
use pallet_assets::{Fungible, IssueAndBurn};
use sp_runtime::{
    traits::{CheckedAdd, CheckedSub, IntegerSquareRoot, SaturatedConversion, Saturating, Scale},
    DispatchError, DispatchResult,
};
use sp_std::{convert::TryInto, prelude::*};

pub type CurrencyOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type BalanceOf<T> = <<T as Config>::FungibleToken as Fungible<
    <T as pallet_assets::Config>::AssetId,
    <T as frame_system::Config>::AccountId,
>>::Balance;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub trait Config:
    frame_system::Config + pallet_assets::Config + pallet_timestamp::Config
{
    type Currency: Currency<Self::AccountId>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type FungibleToken: IssueAndBurn<Self::AssetId, Self::AccountId>;

    type MinimumLiquidity: Get<Self::Balance>;
}

decl_event! {
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        Balance = BalanceOf<T>,
    {
        Mint(AccountId, Balance, Balance),
        Burn(AccountId, Balance, Balance, AccountId),
        SyncDone(Balance, Balance),
        Swap(AccountId, Balance, Balance, Balance, Balance, AccountId),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        OverFlow,
        NotQualifiedMint,
        NotQualifiedBurn,
        NotEnoughLiquidity,
        TooLow,
        InsufficientLiquidityMinted,
        InsufficientLiquidityBurned,
        InsufficientOutputAmount,
        InsufficientInputAmount,
        InsufficientLiquidity,
        InvalidTo,
        InvalidK,
    }
}

decl_storage! {
    trait Store for Module<T: Config> as UniswapExchanges {

        pub AssetId get(fn social_token_id): T::AssetId = 1u32.into();

        pub FeeTo get(fn fee_to): T::AccountId;
        pub Address0 get(fn address0): T::AccountId;
        pub Treasury get(fn treasury): T::AccountId;

        pub Token0 get(fn token0): T::AccountId;
        pub Token1 get(fn token1): T::AccountId;

        pub Reserve0 get(fn reserve0): BalanceOf<T>;
        pub Reserve1 get(fn reserve1): BalanceOf<T>;

        pub Price0CumulativeLast get(fn price0_cumulative_last): u128;
        pub Price1CumulativeLast get(fn price1_cumulative_last): u128;

        pub KLast get(fn k_last): BalanceOf<T>;
        pub BlockTimestampLast get(fn block_timestamp_last): u32;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        fn initialize(origin, fee_to: T::AccountId, address0: T::AccountId, treasury: T::AccountId, token0: T::AccountId, token1: T::AccountId) {
            ensure_root(origin)?;
            <FeeTo<T>>::put(fee_to);
            <Address0<T>>::put(address0);
            <Treasury<T>>::put(treasury);
            <Token0<T>>::put(token0);
            <Token1<T>>::put(token1);
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        fn mint(origin, to: T::AccountId) -> Result<(), DispatchError> {
            let sender = ensure_signed(origin)?;
            let social_token_id = Self::social_token_id();
            let reserve0 = Self::reserve0();
            let reserve1 = Self::reserve1();
            let balance0 = T::FungibleToken::balances(&social_token_id, &Self::token0());
            let balance1 = T::FungibleToken::balances(&social_token_id, &Self::token1());

            let amount0 = balance0.checked_sub(&reserve0).ok_or(Error::<T>::NotEnoughLiquidity)?;
            let amount1 = balance1.checked_sub(&reserve1).ok_or(Error::<T>::NotEnoughLiquidity)?;

            let fee_on = Self::mint_fee(reserve0, reserve1)?;
            let total_supply = T::FungibleToken::total_supply(&social_token_id);
            let liquidity = if total_supply == 0u32.into() {
				let min_liquidity = T::MinimumLiquidity::get().saturated_into::<u128>().saturated_into();
                let liquidity = amount0
                    .saturating_mul(amount1)
                    .integer_sqrt()
                    .saturating_sub(min_liquidity);
                T::FungibleToken::issue(&social_token_id, &Self::address0(), min_liquidity)?;
                liquidity
            } else {
                (amount0.saturating_mul(total_supply) / reserve0).min(amount1.saturating_mul(total_supply) / reserve1)
            };
            ensure!(liquidity > 0u32.into(), Error::<T>::InsufficientLiquidityMinted);
            T::FungibleToken::issue(&social_token_id, &to, liquidity)?;

            let _ = Self::update(balance0, balance1, reserve0, reserve1);
            if fee_on {
                <KLast<T>>::put(reserve0.saturating_mul(reserve1));
            }

            Self::deposit_event(RawEvent::Mint(sender, amount0, amount1));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        fn burn(origin, to: T::AccountId) -> Result<(), DispatchError> {
            let sender = ensure_signed(origin)?;
            let social_token_id = Self::social_token_id();
            let reserve0 = Self::reserve0();
            let reserve1 = Self::reserve1();
            let balance0 = T::FungibleToken::balances(&social_token_id, &Self::token0());
            let balance1 = T::FungibleToken::balances(&social_token_id, &Self::token1());
            let liquidity = <T as Config>::Currency::free_balance(&Self::treasury())
                .saturated_into::<u128>()
                .saturated_into::<BalanceOf<T>>();

            let fee_on = Self::mint_fee(reserve0, reserve1)?;
            let total_supply = T::FungibleToken::total_supply(&social_token_id);


            let amount0 = liquidity.saturating_mul(balance0) / total_supply;
            let amount1 = liquidity.saturating_mul(balance1) / total_supply;
            ensure!(amount0 > 0u32.into() && amount1 > 0u32.into(),  Error::<T>::InsufficientLiquidityBurned);
            T::FungibleToken::burn(&social_token_id, &to, liquidity)?;

			T::FungibleToken::transfer(&social_token_id, &Self::token0(), &to, amount0)?;
			T::FungibleToken::transfer(&social_token_id, &Self::token1(), &to, amount1)?;

            let balance0 = T::FungibleToken::balances(&social_token_id, &Self::token0());
            let balance1 = T::FungibleToken::balances(&social_token_id, &Self::token1());

            let _ = Self::update(balance0, balance1, reserve0, reserve1);
            if fee_on {
                <KLast<T>>::put(reserve0.saturating_mul(reserve1));
            }

            Self::deposit_event(RawEvent::Burn(sender, amount0, amount1, to));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        fn swap(origin, amount0_out: BalanceOf<T>, amount1_out: BalanceOf<T>, to: T::AccountId, data: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(amount0_out > 0u32.into() || amount1_out > 0u32.into(), Error::<T>::InsufficientOutputAmount);
            let social_token_id = Self::social_token_id();
            let reserve0 = Self::reserve0();
            let reserve1 = Self::reserve1();
            ensure!(amount0_out < reserve0 || amount1_out < reserve1, Error::<T>::InsufficientLiquidity);
            let token0 = Self::token0();
            let token1 = Self::token1();
            ensure!(to != token0 && to != token1, Error::<T>::InvalidTo);

            if amount0_out > 0u32.into() {
				T::FungibleToken::transfer(&social_token_id, &token0, &to, amount0_out)?;
            }
            if amount1_out > 0u32.into() {
				T::FungibleToken::transfer(&social_token_id, &token1, &to, amount1_out)?;
            }
            // TODO:
            // if (data.length > 0) IUniswapV2Callee(to).uniswapV2Call(msg.sender, amount0Out, amount1Out, data);
            let balance0 = T::FungibleToken::balances(&social_token_id, &Self::token0());
            let balance1 = T::FungibleToken::balances(&social_token_id, &Self::token1());

            let amount0_in = if balance0 > reserve0 - amount0_out {
                balance0 - (reserve0 - amount0_out)
            } else {
                0u32.into()
            };
            let amount1_in = if balance1 > reserve1 - amount1_out {
                balance1 - (reserve1 - amount1_out)
            } else {
                0u32.into()
            };
            ensure!(amount0_in > 0u32.into() || amount1_in > 0u32.into(), Error::<T>::InsufficientInputAmount);

            let balance0_adjusted = balance0.saturating_mul(1000u32.into()).saturating_sub(amount0_in.saturating_mul(3u32.into()));
            let balance1_adjusted = balance1.saturating_mul(1000u32.into()).saturating_sub(amount1_in.saturating_mul(3u32.into()));
            ensure!(balance0_adjusted.saturating_mul(balance1_adjusted) >= reserve0.saturating_mul(reserve1).saturating_mul(1_000_000u32.into()), Error::<T>::InvalidK);

            let _ = Self::update(balance0, balance1, reserve0, reserve1);

            Self::deposit_event(RawEvent::Swap(sender, amount0_in, amount1_in, amount0_out, amount1_out, to));

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    fn mint_fee(reserve0: BalanceOf<T>, reserve1: BalanceOf<T>) -> Result<bool, DispatchError> {
        let fee_to = Self::fee_to();
        let fee_on = fee_to != Self::address0();
        let k_last = Self::k_last();
        if fee_on {
            if k_last != 0u32.into() {
                let root_k = (reserve0 * reserve1)
                    .saturated_into::<u128>()
                    .integer_sqrt();
                let root_k_last = k_last.saturated_into::<u128>().integer_sqrt();
                if root_k > root_k_last {
                    let total_supply = T::FungibleToken::total_supply(&Self::social_token_id())
                        .saturated_into::<u128>();
                    let numerator = total_supply * (root_k - root_k_last);
                    let denominator = root_k * 5 + root_k_last;
                    let liquidity = numerator / denominator;
                    if liquidity > 0 {
                        T::FungibleToken::issue(
                            &Self::social_token_id(),
                            &fee_to,
                            liquidity.saturated_into(),
                        )?;
                    }
                }
            }
        } else if k_last != 0u32.into() {
            <KLast<T>>::put(Into::<BalanceOf<T>>::into(0u32));
        }

        Ok(fee_on)
    }

    fn update(
        balance0: BalanceOf<T>,
        balance1: BalanceOf<T>,
        reserve0: BalanceOf<T>,
        reserve1: BalanceOf<T>,
    ) -> DispatchResult {
        let balance0_copy = balance0.clone();
        let balance1_copy = balance1.clone();
        ensure!(
            balance0_copy
                .checked_add(&Into::<BalanceOf<T>>::into(1u32))
                .is_some(),
            Error::<T>::OverFlow
        );
        ensure!(
            balance1_copy
                .checked_add(&Into::<BalanceOf<T>>::into(1u32))
                .is_some(),
            Error::<T>::OverFlow
        );
        let block_timestamp = TryInto::<u32>::try_into(
            <pallet_timestamp::Module<T>>::now()
                .saturated_into::<u64>()
                .rem(2u64.pow(32)),
        )
        .map_err(|_| Error::<T>::OverFlow)?;
        let time_elapsed = block_timestamp - Self::block_timestamp_last();
        if time_elapsed > 0u32 && reserve0 != 0u32.into() && reserve1 != 0u32.into() {
            // * never overflows, and + overflow is desired
            <Price0CumulativeLast>::mutate(|price| {
                let new_price = (reserve1
                    .saturated_into::<u128>()
                    .saturating_mul(2u128.pow(64))
                    / reserve0.saturated_into::<u128>())
                .saturating_mul(Into::<u128>::into(time_elapsed));
                *price = price.saturating_add(new_price);
            });
            <Price1CumulativeLast>::mutate(|price| {
                let new_price = (reserve0
                    .saturated_into::<u128>()
                    .saturating_mul(2u128.pow(64))
                    / reserve1.saturated_into::<u128>())
                .saturating_mul(Into::<u128>::into(time_elapsed));
                *price = price.saturating_add(new_price);
            });
        }

        <Reserve0<T>>::put(balance0);
        <Reserve1<T>>::put(balance1);
        <BlockTimestampLast>::put(block_timestamp);

        Self::deposit_event(RawEvent::SyncDone(reserve0, reserve1));

        Ok(())
    }

    fn _safe_transfer(
        _token: &T::AccountId,
        to: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        <T as Config>::Currency::transfer(
            &Self::treasury(),
            to,
            amount.saturated_into::<u128>().saturated_into(),
            ExistenceRequirement::AllowDeath,
        )?;
        Ok(())
    }
}
