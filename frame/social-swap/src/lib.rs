//! This Module serves as a combination of uniswap factory
//! and exchange
//!
//! Now only support add CustomToken-NativeToken pair.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Currency, ExistenceRequirement, Get},
    IterableStorageDoubleMap, IterableStorageMap, Parameter,
};
use frame_system::ensure_signed;
use pallet_assets::{Fungible, IssueAndBurn, TokenDossier};
use sp_runtime::{
    traits::{
        AccountIdConversion, AtLeast32Bit, AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedDiv,
        CheckedMul, CheckedSub, Convert, MaybeSerializeDeserialize, Member, One,
        SaturatedConversion, Saturating, UniqueSaturatedInto, Zero,
    },
    DispatchError, DispatchResult, ModuleId, Perbill, RuntimeDebug,
};
use sp_std::{cmp, convert::TryFrom, fmt::Debug, result};
use sp_std::{ops::Div, prelude::*};

type TokenDossierOf = TokenDossier;
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

#[derive(Encode, Decode)]
pub struct Exchange<AssetId, Balance, CurrencyOf> {
    pub lp_token: AssetId,
    pub trade_token: AssetId,
    /// trade token amount in the exchange
    pub trade_token_amount: Balance,
    /// native currency amount in the exchange
    pub native_token_amount: CurrencyOf,
}

impl<A, B: Zero, C: Zero> Exchange<A, B, C> {
    fn new(lp_token: A, trade_token: A) -> Self {
        Self {
            lp_token,
            trade_token,
            trade_token_amount: Zero::zero(),
            native_token_amount: Zero::zero(),
        }
    }
}

pub trait Config: frame_system::Config + pallet_assets::Config {
    type Currency: Currency<Self::AccountId>;
    type ModuleId: Get<ModuleId>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type ExchangeId: Parameter + Member + AtLeast32Bit + Default + Copy;
    type FungibleToken: IssueAndBurn<Self::AssetId, Self::AccountId>;
    /// help to convert native token balance to fungible token balance
    type Handler: Convert<CurrencyOf<Self>, BalanceOf<Self>>;
}

decl_event! {
    pub enum Event<T> where
        CurrencyOf = CurrencyOf<T>,
        <T as frame_system::Config>::AccountId,
        Balance = BalanceOf<T>,
        <T as Config>::ExchangeId,
        <T as pallet_assets::Config>::AssetId,

    {
        /// A new exchange created. [exchange_id, lp_token_id, trade_token_id]
        NewExchange(ExchangeId, AssetId, AssetId),
        /// Add liquidity [exchange_id, liquidity_provider, native_token, trade_token, liquidity_minted]
        AddLiquidity(ExchangeId, AccountId, CurrencyOf, Balance, Balance),
        /// Remove liquidity [exchange_id, liquidity_burner, native_token, trade_token, liquidity_burned]
        RemoveLiquidity(ExchangeId, AccountId, CurrencyOf, Balance, Balance),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        TooLate,
        /// exchange already exists
        ExchangeExists,
        /// exchange does not exist
        ExchangeNotExists,
        TradeTokenNotExists,
        OverFlow,
        /// trade tokens transferred are too low,
        /// or liquidity minted is too low
        NotQualifiedMint,
        NotQualifiedBurn,
        NotEnoughLiquidity,
        TooLow,
    }
}

decl_storage! {
    trait Store for Module<T: Config> as UniswapExchanges {

        pub Exchanges get(fn exchanges): map hasher(twox_64_concat) T::ExchangeId => Option<Exchange<T::AssetId, BalanceOf<T>, CurrencyOf<T>>>;

        pub TradeTokenToExchange get(fn tt_to_exchange): map hasher(twox_64_concat) T::AssetId => Option<T::ExchangeId>;

        pub LPTokenToExchange get(fn lp_to_exchange): map hasher(twox_64_concat) T::AssetId => T::ExchangeId;
        /// The next exchange identifier
        pub NextExchangeId get(fn next_exchange_id): T::ExchangeId;

    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        #[weight = 0]
        fn create_exchange(origin, token_id: T::AssetId) -> DispatchResult {
            // make sure there will be only one exchange for a specific trade token
            // and this trade token exists
            ensure!(T::FungibleToken::exists(&token_id), Error::<T>::TradeTokenNotExists);

            ensure!(Self::tt_to_exchange(&token_id).is_none(), Error::<T>::ExchangeExists);

            // new allocated exchange id, and craete a new lp token for it
            let exchange_id = Self::next_exchange_id();
            let lp_asset_id = Self::create_lp_token(exchange_id)?;

            let exchange_info = Exchange::new(lp_asset_id, token_id);
            // add new exchange info
            <Exchanges<T>>::insert(&exchange_id, exchange_info);
            <TradeTokenToExchange<T>>::insert(&token_id, exchange_id);
            <LPTokenToExchange<T>>::insert(&lp_asset_id, exchange_id);

            Self::deposit_event(RawEvent::NewExchange(exchange_id, lp_asset_id, token_id));

            Ok(())
        }


        /// Deposit currency(native token) and trade tokens at current ratio to mint lp tokens.
        /// One thing different from eth uniswap is `deadline` here metered in BlockNumber
        #[weight = 0]
        fn add_liquidity(
            origin,
            exchange_id: T::ExchangeId,
            native_token_transferred: CurrencyOf<T>,
            min_liquidity: BalanceOf<T>,
            max_trade_tokens: BalanceOf<T>,
            deadline: T::BlockNumber
        ) {

            let who = ensure_signed(origin)?;
            let mut exchange = Self::exchanges(&exchange_id).unwrap();
            let lp_token_id = exchange.lp_token;
            let trade_token_id = exchange.trade_token;
            ensure!(deadline > <frame_system::Module<T>>::block_number(), Error::<T>::TooLate);
            // make sure that the exchange has been created
            ensure!(Self::exchanges(&exchange_id).is_some() &&
                !T::FungibleToken::total_supply(&trade_token_id).is_zero(), Error::<T>::TradeTokenNotExists);

            // current total liquidity, refer to l_0 in spec
            let lp_total_supply = T::FungibleToken::total_supply(&lp_token_id);
            if lp_total_supply > Zero::zero() {
                // amount of DOT balance in this exchange, e_0 in spec
                let native_token_balance = exchange.native_token_amount;
                // amount of the trade token balance in the exchange, t_0 in spec
                let trade_token_balance = exchange.trade_token_amount;
                // amount of trade token that should be transferred, refer to αt_0 + 1
                // which is (∆e / e_0) * t_0 + 1
                let trade_token_amount = T::Handler::convert(native_token_transferred)
                    .checked_mul(&trade_token_balance)
                    .ok_or(Error::<T>::OverFlow)?
                    .div(T::Handler::convert(native_token_balance))
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::OverFlow)?;
                // liquidity minted, refer to (∆e * l_0) / e_0 in sepc
                let liquidity_minted = T::Handler::convert(native_token_transferred)
                    .checked_mul(&lp_total_supply)
                    .ok_or(Error::<T>::OverFlow)?
                    .div(T::Handler::convert(native_token_balance));

                ensure!(max_trade_tokens >= trade_token_amount &&
                    liquidity_minted >= min_liquidity, Error::<T>::NotQualifiedMint);

                Self::input_liquidity(&who, native_token_transferred, &trade_token_id, trade_token_amount, &mut exchange)?;
                <Exchanges<T>>::mutate(&exchange_id, |e| *e = Some(exchange));
                // add lp token to the miner, also add the total supply
                T::FungibleToken::issue(&lp_token_id, &who, liquidity_minted);
                Self::deposit_event(RawEvent::AddLiquidity(
                    exchange_id, who.clone(), native_token_transferred,	trade_token_amount, liquidity_minted)
                );
            } else {
                // if liquidity = 0
                let initial_liquidity = T::Handler::convert(native_token_transferred);
                T::FungibleToken::issue(&lp_token_id, &who, initial_liquidity)?;

                Self::input_liquidity(&who, native_token_transferred, &trade_token_id, max_trade_tokens, &mut exchange)?;
                <Exchanges<T>>::mutate(&exchange_id, |e| *e = Some(exchange));
                Self::deposit_event(RawEvent::AddLiquidity(
                    exchange_id, who.clone(), native_token_transferred,	max_trade_tokens, initial_liquidity)
                );
            }
        }

        #[weight = 0]
        fn remove_liquidity(
            origin,
            exchange_id: T::ExchangeId,
            liquidity_burned: BalanceOf<T>,
            min_native_tokens: CurrencyOf<T>,
            min_trade_tokens: BalanceOf<T>,
            deadline: T::BlockNumber
        ) {
            let who = ensure_signed(origin)?;
            let mut exchange = Self::exchanges(&exchange_id).unwrap();
            let lp_token_id = exchange.lp_token;
            let trade_token_id = exchange.trade_token;

            ensure!(deadline > <frame_system::Module<T>>::block_number(), Error::<T>::TooLate);
            ensure!(Self::exchanges(&exchange_id).is_some() &&
                !T::FungibleToken::total_supply(&trade_token_id).is_zero(), Error::<T>::TradeTokenNotExists);
            ensure!(liquidity_burned > Zero::zero() && min_native_tokens > Zero::zero() &&
                min_trade_tokens > Zero::zero(), Error::<T>::NotQualifiedBurn);

            let total_liquidity = T::FungibleToken::total_supply(&lp_token_id);
            ensure!(total_liquidity > Zero::zero(), Error::<T>::NotEnoughLiquidity);

            let mut exchange = Self::exchanges(&exchange_id).unwrap();
            // t_0
            let trade_token_balance = exchange.trade_token_amount;
            // e_0
            let native_token_balance = exchange.native_token_amount;
            let native_token_should_transfer: u128 = liquidity_burned.checked_mul(&trade_token_balance)
                .ok_or(Error::<T>::OverFlow)?
                .div(total_liquidity).unique_saturated_into();

            // convert it into CurrencyOf
            let native_token_should_transfer: CurrencyOf<T> = native_token_should_transfer.saturated_into();
            let trade_token_should_transfer = liquidity_burned
                .checked_mul(&trade_token_balance)
                .ok_or(Error::<T>::OverFlow)?
                .div(total_liquidity);

            ensure!(native_token_should_transfer > min_native_tokens &&
                trade_token_should_transfer > min_trade_tokens, Error::<T>::TooLow);

            // burn liquidity
            T::FungibleToken::burn(&lp_token_id, &who, liquidity_burned)?;
            Self::withdraw_liquidity(
                &who, native_token_should_transfer, &trade_token_id, trade_token_should_transfer, &mut exchange)?;

            Self::deposit_event(
                RawEvent::RemoveLiquidity(exchange_id, who.clone(), native_token_should_transfer, trade_token_should_transfer, liquidity_burned));
        }

        #[weight = 0]
        fn native_to_trade_token_input(
            origin,
            exchange_id: T::ExchangeId,
            native_sold: CurrencyOf<T>,
            min_trade_tokens: BalanceOf<T>,
            deadline: T::BlockNumber,
            recipient: T::AccountId
        ) {
            let buyer = ensure_signed(origin)?;
            let mut exchange = Self::exchanges(&exchange_id).ok_or(Error::<T>::ExchangeNotExists)?;
            Self::native_to_trade_input(&mut exchange, native_sold, min_trade_tokens, deadline, &buyer, &recipient);
        }

        #[weight = 0]
        fn trade_to_native_token_input(
            origin,
            exchange_id: T::ExchangeId,
            trade_token_sold: BalanceOf<T>,
            min_native_tokens: CurrencyOf<T>,
            deadline: T::BlockNumber,
            recipient: T::AccountId
        ) {
            let buyer = ensure_signed(origin)?;
            let mut exchange = Self::exchanges(&exchange_id).ok_or(Error::<T>::ExchangeNotExists)?;
            Self::trade_to_native_input(&mut exchange, trade_token_sold, min_native_tokens, deadline, &buyer, &recipient);
        }

        #[weight = 0]
        fn native_to_trade_token_output(
            origin,
            exchange_id: T::ExchangeId,
            trade_token_bought: BalanceOf<T>,
            deadline: T::BlockNumber,
            recipient: T::AccountId
        ) {
            let buyer = ensure_signed(origin)?;
            let mut exchange = Self::exchanges(&exchange_id).ok_or(Error::<T>::ExchangeNotExists)?;
            Self::native_to_trade_output(&mut exchange, trade_token_bought, deadline, &buyer, &recipient);
        }

        #[weight = 0]
        fn trade_to_native_token_output(
            origin,
            exchange_id: T::ExchangeId,
            native_token_bought:CurrencyOf<T>,
            deadline: T::BlockNumber,
            recipient: T::AccountId
        ) {
            let buyer = ensure_signed(origin)?;
            let mut exchange = Self::exchanges(&exchange_id).ok_or(Error::<T>::ExchangeNotExists)?;
            Self::trade_to_native_output(&mut exchange, native_token_bought, deadline, &buyer, &recipient);
        }

    }
}

impl<T: Config> Module<T> {
    /// The account id of the exchanges pot
    fn account_id() -> T::AccountId {
        T::ModuleId::get().into_account()
    }

    fn create_lp_token(exchange_id: T::ExchangeId) -> Result<T::AssetId, DispatchError> {
        // create a new lp token for exchange
        T::FungibleToken::create_new_asset(&Self::account_id(), TokenDossierOf::new_lp_token())
    }

    fn input_liquidity(
        who: &T::AccountId,
        native_token_amount: CurrencyOf<T>,
        trade_token_id: &T::AssetId,
        trade_token_amount: BalanceOf<T>,
        exchange: &mut Exchange<T::AssetId, BalanceOf<T>, CurrencyOf<T>>,
    ) -> DispatchResult {
        let this = Self::account_id();
        <T as Config>::Currency::transfer(
            who,
            &this,
            native_token_amount,
            ExistenceRequirement::KeepAlive,
        )?;
        exchange.native_token_amount += native_token_amount;
        T::FungibleToken::transfer(&trade_token_id, who, &this, trade_token_amount)?;
        exchange.trade_token_amount += trade_token_amount;

        Ok(())
    }

    fn withdraw_liquidity(
        who: &T::AccountId,
        native_token_amount: CurrencyOf<T>,
        trade_token_id: &T::AssetId,
        trade_token_amount: BalanceOf<T>,
        exchange: &mut Exchange<T::AssetId, BalanceOf<T>, CurrencyOf<T>>,
    ) -> DispatchResult {
        let this = Self::account_id();
        <T as Config>::Currency::transfer(
            &this,
            who,
            native_token_amount,
            ExistenceRequirement::KeepAlive,
        )?;
        exchange.native_token_amount -= native_token_amount;
        T::FungibleToken::transfer(&trade_token_id, &this, who, trade_token_amount)?;
        exchange.trade_token_amount -= trade_token_amount;

        Ok(())
    }

    /// change the storage
    /// put this at the end of the `native_to_trade_input` function
    fn native_to_trade_swap(
        exchange: &mut Exchange<T::AssetId, BalanceOf<T>, CurrencyOf<T>>,
        native_in: CurrencyOf<T>,
        trade_token_id: &T::AssetId,
        trade_out: BalanceOf<T>,
        buyer: &T::AccountId,
        recipient: &T::AccountId,
    ) -> DispatchResult {
        let this = Self::account_id();
        // transfer native token in
        <T as Config>::Currency::transfer(&buyer, &this, native_in, ExistenceRequirement::KeepAlive)?;
        // modify Exchange
        exchange.native_token_amount += native_in;
        T::FungibleToken::transfer(&trade_token_id, &this, recipient, trade_out)?;
        exchange.trade_token_amount -= trade_out;
        Ok(())
    }

    /// change the storage
    fn trade_to_native_swap(
        exchange: &mut Exchange<T::AssetId, BalanceOf<T>, CurrencyOf<T>>,
        trade_in: BalanceOf<T>,
        trade_token_id: &T::AssetId,
        native_out: CurrencyOf<T>,
        buyer: &T::AccountId,
        recipient: &T::AccountId,
    ) -> DispatchResult {
        let this = Self::account_id();
        // transfer native token in
        <T as Config>::Currency::transfer(&this, &buyer, native_out, ExistenceRequirement::KeepAlive)?;
        // modify Exchange
        exchange.native_token_amount -= native_out;
        T::FungibleToken::transfer(&trade_token_id, recipient, &this, trade_in)?;
        exchange.trade_token_amount += trade_in;
        Ok(())
    }

    // TODO: not safe
    fn input_price_u128(input_amount: u128, input_reserve: u128, output_reserve: u128) -> u128 {
        let input_amount_with_fee = input_amount * 997;
        let numerator = input_amount_with_fee * output_reserve;
        let demonator = (input_reserve * 1000) + input_amount_with_fee;
        numerator / demonator
    }

    fn output_price_u128(output_amount: u128, input_reserve: u128, output_reserve: u128) -> u128 {
        let numerator = input_reserve * output_amount * 1000;
        let denominator = (output_reserve - output_amount) * 997;
        numerator / denominator + 1
    }

    fn native_to_trade_input(
        exchange: &mut Exchange<T::AssetId, BalanceOf<T>, CurrencyOf<T>>,
        native_in: CurrencyOf<T>,
        min_trade_tokens: BalanceOf<T>,
        deadline: T::BlockNumber,
        buyer: &T::AccountId,
        recipient: &T::AccountId,
    ) -> DispatchResult {
        ensure!(
            deadline >= <frame_system::Module<T>>::block_number(),
            Error::<T>::TooLate
        );
        ensure!(
            native_in > Zero::zero() && min_trade_tokens > Zero::zero(),
            Error::<T>::TooLow
        );
        let trade_token_reserve = exchange.trade_token_amount;
        let native_token_reserve = exchange.native_token_amount;
        let trade_token_bought = Self::input_price_u128(
            native_in.unique_saturated_into(),
            native_token_reserve.unique_saturated_into(),
            trade_token_reserve.unique_saturated_into(),
        );
        let trade_token_bought = <BalanceOf<T>>::saturated_from(trade_token_bought);

        let trade_token_id = exchange.trade_token;
        Self::native_to_trade_swap(
            exchange,
            native_in,
            &trade_token_id,
            trade_token_bought,
            buyer,
            recipient,
        )
    }

    fn trade_to_native_input(
        exchange: &mut Exchange<T::AssetId, BalanceOf<T>, CurrencyOf<T>>,
        trade_in: BalanceOf<T>,
        min_native_tokens: CurrencyOf<T>,
        deadline: T::BlockNumber,
        buyer: &T::AccountId,
        recipient: &T::AccountId,
    ) -> DispatchResult {
        ensure!(
            deadline >= <frame_system::Module<T>>::block_number(),
            Error::<T>::TooLate
        );
        ensure!(
            trade_in > Zero::zero() && min_native_tokens > Zero::zero(),
            Error::<T>::TooLow
        );

        let trade_reserve = exchange.trade_token_amount;
        let native_reserve = exchange.native_token_amount;
        let native_token_bought = Self::input_price_u128(
            trade_in.unique_saturated_into(),
            trade_reserve.unique_saturated_into(),
            native_reserve.unique_saturated_into(),
        );
        let native_token_bought = <CurrencyOf<T>>::saturated_from(native_token_bought);
        let trade_token_id = exchange.trade_token;
        Self::trade_to_native_swap(
            exchange,
            trade_in,
            &trade_token_id,
            native_token_bought,
            buyer,
            recipient,
        )
    }

    /// evoked by dispatchble functions
    fn native_to_trade_output(
        exchange: &mut Exchange<T::AssetId, BalanceOf<T>, CurrencyOf<T>>,
        trade_token_bought: BalanceOf<T>,
        deadline: T::BlockNumber,
        buyer: &T::AccountId,
        recipient: &T::AccountId,
    ) -> DispatchResult {
        ensure!(
            deadline >= <frame_system::Module<T>>::block_number(),
            Error::<T>::TooLate
        );
        ensure!(trade_token_bought > Zero::zero(), Error::<T>::TooLow);

        let native_reserve = exchange.native_token_amount;
        let trade_token_reserve = exchange.trade_token_amount;
        let trade_token_id = exchange.trade_token;
        let native_sold = Self::output_price_u128(
            trade_token_bought.unique_saturated_into(),
            native_reserve.unique_saturated_into(),
            trade_token_reserve.unique_saturated_into(),
        );

        let native_sold = <CurrencyOf<T>>::saturated_from(native_sold);

        Self::native_to_trade_swap(
            exchange,
            native_sold,
            &trade_token_id,
            trade_token_bought,
            buyer,
            recipient,
        )
    }

    /// evoked by dispatchable functions
    fn trade_to_native_output(
        exchange: &mut Exchange<T::AssetId, BalanceOf<T>, CurrencyOf<T>>,
        native_token_bought: CurrencyOf<T>,
        deadline: T::BlockNumber,
        buyer: &T::AccountId,
        recipient: &T::AccountId,
    ) -> DispatchResult {
        ensure!(
            deadline >= <frame_system::Module<T>>::block_number(),
            Error::<T>::TooLate
        );
        ensure!(native_token_bought > Zero::zero(), Error::<T>::TooLow);

        let native_reserve = exchange.native_token_amount;
        let trade_token_reserve = exchange.trade_token_amount;
        let trade_token_id = exchange.trade_token;

        let trade_tokens_sold = Self::output_price_u128(
            native_token_bought.unique_saturated_into(),
            trade_token_reserve.unique_saturated_into(),
            native_reserve.unique_saturated_into(),
        );

        let trade_tokens_sold = <BalanceOf<T>>::saturated_from(trade_tokens_sold);

        Self::trade_to_native_swap(
            exchange,
            trade_tokens_sold,
            &trade_token_id,
            native_token_bought,
            buyer,
            recipient,
        )
    }
}
