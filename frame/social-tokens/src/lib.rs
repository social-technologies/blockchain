#![cfg_attr(not(feature = "std"), no_std)]

pub use self::imbalances::{NegativeImbalance, PositiveImbalance};
use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{
        EnsureOrigin, ExistenceRequirement, ExistenceRequirement::AllowDeath, Get, Imbalance,
        LockIdentifier, OnNewAccount, StoredMap, TryDrop, WithdrawReasons,
    },
    Parameter,
};
use frame_system::{ensure_signed, split_inner, RefCount};
use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, Member, Saturating, StaticLookup,
        StoredMapError, Zero,
    },
    DispatchError, RuntimeDebug, SaturatedConversion,
};
use sp_std::prelude::*;
use sp_std::{cmp, convert::Infallible, ops::BitOr, result};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    /// The balance of an account.
    type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

    type SocialTokenId: Parameter + AtLeast32BitUnsigned + Default + Copy;

    /// The minimum amount required to keep an account open.
    type ExistentialDeposit: Get<Self::Balance>;

    /// Handler for when a new account has just been created.
    type OnNewAccount: OnNewAccount<(Self::SocialTokenId, Self::AccountId)>;

    /// Maximum Supply of the Social Token.
    type MaxSocialTokensSupply: Get<u128>;

    /// Origin from which can create a new social.
    type SocialCreatorOrigin: EnsureOrigin<Self::Origin>;
}

/// Simplified reasons for withdrawing balance.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
pub enum Reasons {
    /// Paying system transaction fees.
    Fee = 0,
    /// Any reason other than paying system transaction fees.
    Misc = 1,
    /// Any reason at all.
    All = 2,
}

impl From<WithdrawReasons> for Reasons {
	fn from(r: WithdrawReasons) -> Reasons {
		if r == WithdrawReasons::from(WithdrawReasons::TRANSACTION_PAYMENT) {
			Reasons::Fee
		} else if r.contains(WithdrawReasons::TRANSACTION_PAYMENT) {
			Reasons::All
		} else {
			Reasons::Misc
		}
	}
}

impl BitOr for Reasons {
    type Output = Reasons;
    fn bitor(self, other: Reasons) -> Reasons {
        if self == other {
            return self;
        }
        Reasons::All
    }
}

/// A single lock on a balance. There can be many of these on an account and they "overlap", so the
/// same balance is frozen by multiple locks.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct BalanceLock<Balance> {
    /// An identifier for this lock. Only one lock may be in existence for each identifier.
    pub id: LockIdentifier,
    /// The amount which the free balance may not drop below when this lock is in effect.
    pub amount: Balance,
    /// If true, then the lock remains in effect even for payment of transaction fees.
    pub reasons: Reasons,
}

/// All balance information for an account.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct AccountData<Balance> {
    /// Non-reserved part of the balance. There may still be restrictions on this, but it is the
    /// total pool what may in principle be transferred, reserved and used for tipping.
    ///
    /// This is the only balance that matters in terms of most operations on tokens. It
    /// alone is used to determine the balance when in the contract execution environment.
    pub free: Balance,
    /// Balance which is reserved and may not be used at all.
    ///
    /// This can still get slashed, but gets slashed last of all.
    ///
    /// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
    /// that are still 'owned' by the account holder, but which are suspendable.
    pub reserved: Balance,
    /// The amount that `free` may not drop below when withdrawing for *anything except transaction
    /// fee payment*.
    pub misc_frozen: Balance,
    /// The amount that `free` may not drop below when withdrawing specifically for transaction
    /// fee payment.
    pub fee_frozen: Balance,
}

impl<Balance: Saturating + Copy + Ord> AccountData<Balance> {
    /// How much this account's balance can be reduced for the given `reasons`.
    #[allow(dead_code)]
    fn usable(&self, reasons: Reasons) -> Balance {
        self.free.saturating_sub(self.frozen(reasons))
    }
    /// The amount that this account's free balance may not be reduced beyond for the given
    /// `reasons`.
    fn frozen(&self, reasons: Reasons) -> Balance {
        match reasons {
            Reasons::All => self.misc_frozen.max(self.fee_frozen),
            Reasons::Misc => self.misc_frozen,
            Reasons::Fee => self.fee_frozen,
        }
    }
    /// The total balance in this account including any that is reserved and ignoring any frozen.
    fn total(&self) -> Balance {
        self.free.saturating_add(self.reserved)
    }
}

type Symbol = [u8; 8];
const NET_LP: Symbol = *b"NETDEX";

/// TODO: consider if there need to be more fields
#[derive(Encode, Decode)]
pub struct TokenDossier {
    pub symbol: Symbol,
}

impl TokenDossier {
    pub fn new_lp_token() -> Self {
        TokenDossier { symbol: NET_LP }
    }
}

decl_storage! {
    trait Store for Module<T: Config> as SocialTokens {
        MaxSocialTokenId get(fn max_social_token_id): T::SocialTokenId = 17u32.into(); // Remove this, anyone can create
        MinSocialTokenId get(fn min_social_token_id): T::SocialTokenId = 1u32.into();

        pub TotalIssuance: map hasher(blake2_128_concat) T::SocialTokenId => T::Balance;

        /// The full account information for a particular account ID.
        pub SystemAccount get(fn system_account):
            map hasher(blake2_128_concat) (T::SocialTokenId, T::AccountId) => AccountInfo<T::Index, AccountData<T::Balance>>;

        /// Any liquidity locks on some account balances.
        /// NOTE: Should only be accessed when setting, changing and freeing a lock.
        pub Locks get(fn locks): map hasher(blake2_128_concat) (T::SocialTokenId, T::AccountId) => Vec<BalanceLock<T::Balance>>;

        TokenInfo get(fn token_info): map hasher(twox_64_concat) T::SocialTokenId => Option<TokenDossier>;
        /// Allowance
        Allowances get(fn allowances):
            double_map  hasher(twox_64_concat) T::SocialTokenId, hasher(blake2_128_concat) (T::AccountId, T::AccountId) => T::Balance;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        SocialTokenId = <T as Config>::SocialTokenId,
        SocialTokenBalance = <T as Config>::Balance,
    {
        /// An account was created with some free balance. \[account, free_balance\]
        Endowed(AccountId, SocialTokenId, SocialTokenBalance),
        /// Some assets were transferred. \[token_id, from, to, amount\]
        Transfer(AccountId, AccountId, SocialTokenId, SocialTokenBalance),
        /// A balance was set by root. \[who, free, reserved\]
        SocialTokenBalanceSet(
            AccountId,
            SocialTokenId,
            SocialTokenBalance,
            SocialTokenBalance,
        ),
        /// Some amount was deposited (e.g. for transaction fees). \[who, deposit\]
        Deposit(AccountId, SocialTokenId, SocialTokenBalance),
        /// Some balance was reserved (moved from free to reserved). \[who, value\]
        Reserved(AccountId, SocialTokenId, SocialTokenBalance),
        /// Some balance was unreserved (moved from reserved to free). \[who, value\]
        Unreserved(AccountId, SocialTokenId, SocialTokenBalance),
        /// A new \[account\] was created.
        NewAccount(AccountId, SocialTokenId),
        SocialTokenCreated(SocialTokenId),
        /// Some assets were issued. [token_id, owner, total_supply]
        Issued(SocialTokenId, AccountId, SocialTokenBalance),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        InvalidSocialTokenId,
        /// Transfer amount should be non-zero
        AmountZero,
        /// Account balance must be greater than or equal to the transfer amount
        BalanceLow,
        /// Vesting balance too high to send value
        VestingBalance,
        /// Account liquidity restrictions prevent withdrawal
        LiquidityRestrictions,
        /// Got an overflow after adding
        Overflow,
        /// Balance too low to send value
        InsufficientBalance,
        /// Value too low to create account due to existential deposit
        ExistentialDeposit,
        /// Transfer/payment would kill account
        KeepAlive,
        /// A vesting schedule already exists for this account
        ExistingVestingSchedule,
        /// Beneficiary account must pre-exist
        DeadAccount,
        InvalidTransfer,
        /// Have no permission to transfer someone's balance
        NotAllowed,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        const MaxSocialTokensSupply: u128 = T::MaxSocialTokensSupply::get();

        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(
            origin,
            #[compact] token_id: T::SocialTokenId,
            target: <T::Lookup as StaticLookup>::Source,
            #[compact] value: T::Balance
        ) {
            let transactor = ensure_signed(origin)?;
            let dest = T::Lookup::lookup(target)?;
            Self::do_transfer(&transactor, &dest, token_id, value, ExistenceRequirement::AllowDeath)?;
        }

        #[weight = 1_000_000_000_000]
        pub fn create_social_token(origin) {
            T::SocialCreatorOrigin::ensure_origin(origin)?;
            let new_social_id = Self::create_new_social_token()?;
            Self::deposit_event(RawEvent::SocialTokenCreated(new_social_id));
        }
    }
}

impl<T: Config> Module<T> {
	fn total_balance(who: &T::AccountId, token_id: T::SocialTokenId) -> T::Balance {
		Self::account(token_id, who).total()
	}

    pub fn balance(who: T::AccountId, token_id: T::SocialTokenId) -> T::Balance {
        Self::free_balance(who, token_id)
    }

    /// Get the free balance of an account.
    pub fn free_balance(
        who: impl sp_std::borrow::Borrow<T::AccountId>,
        token_id: T::SocialTokenId,
    ) -> T::Balance {
        Self::account(token_id, who.borrow()).free
    }

    /// Get the reserved balance of an account.
    pub fn reserved_balance(
        who: impl sp_std::borrow::Borrow<T::AccountId>,
        token_id: T::SocialTokenId,
    ) -> T::Balance {
        Self::account(token_id, who.borrow()).reserved
    }

    pub fn mint_social_token(target: T::AccountId, token_id: T::SocialTokenId, value: T::Balance) {
        if value.is_zero() {
            return;
        }
        let current_balance = Self::free_balance(&target, token_id);
        let total_supply = T::MaxSocialTokensSupply::get().saturated_into();
        let allowed_value = if current_balance + value > total_supply {
            total_supply.saturating_sub(current_balance.into())
        } else {
            value
        };

        Self::mutate(&(token_id, target.clone()), |account_data| {
            account_data.free = account_data
                .free
                .checked_add(&allowed_value)
                .unwrap_or_else(|| T::MaxSocialTokensSupply::get().saturated_into())
        });
    }

    pub fn burn_social_token(target: T::AccountId, token_id: T::SocialTokenId, value: T::Balance) {
        if value.is_zero() {
            return;
        }
        let current_balance = Self::free_balance(&target, token_id);
        let ed = T::ExistentialDeposit::get();
        let allowed_value = if current_balance - value < ed {
            current_balance - ed
        } else {
            value
        };

        Self::mutate(&(token_id, target.clone()), |account_data| {
            account_data.free = account_data
                .free
                .checked_sub(&allowed_value)
                .unwrap_or_else(|| ed)
        });
    }

    pub fn validate_social_token_id(token_id: T::SocialTokenId) -> DispatchResult {
        ensure!(
            token_id >= <MinSocialTokenId<T>>::get() && token_id <= <MaxSocialTokenId<T>>::get(),
            Error::<T>::InvalidSocialTokenId
        );

        Ok(())
    }

    pub fn social_token_ids() -> (T::SocialTokenId, T::SocialTokenId) {
        (<MinSocialTokenId<T>>::get(), <MaxSocialTokenId<T>>::get())
    }

    // Transfer some free balance from `transactor` to `dest`, respecting existence requirements.
    // Is a no-op if value to be transferred is zero or the `transactor` is the same as `dest`.
    pub fn do_transfer(
        transactor: &T::AccountId,
        dest: &T::AccountId,
        token_id: T::SocialTokenId,
        value: T::Balance,
        existence_requirement: ExistenceRequirement,
    ) -> DispatchResult {
        if value.is_zero() || transactor == dest {
            return Ok(());
        }

        Self::try_mutate_account(dest, token_id, |to_account, _| -> DispatchResult {
            Self::try_mutate_account(transactor, token_id, |from_account, _| -> DispatchResult {
                from_account.free = from_account
                    .free
                    .checked_sub(&value)
                    .ok_or(Error::<T>::InsufficientBalance)?;

                // NOTE: total stake being stored in the same type means that this could never overflow
                // but better to be safe than sorry.
                to_account.free = to_account
                    .free
                    .checked_add(&value)
                    .ok_or(Error::<T>::Overflow)?;

                let ed = T::ExistentialDeposit::get();
                ensure!(to_account.total() >= ed, Error::<T>::ExistentialDeposit);

                Self::ensure_can_withdraw(
                    transactor,
                    token_id,
                    value,
                    WithdrawReasons::TRANSFER,
                    from_account.free,
                )?;

                let allow_death = existence_requirement == ExistenceRequirement::AllowDeath;
                let allow_death = allow_death && frame_system::Module::<T>::allow_death(transactor);
                ensure!(
                    allow_death || from_account.free >= ed,
                    Error::<T>::KeepAlive
                );

                Ok(())
            })
        })?;

        // Emit transfer event.
        Self::deposit_event(RawEvent::Transfer(
            transactor.clone(),
            dest.clone(),
            token_id,
            value,
        ));

        Ok(())
    }

    fn create_new_social_token() -> Result<T::SocialTokenId, DispatchError> {
        let new_social_id = <MaxSocialTokenId<T>>::get()
            .checked_add(&1u32.into())
            .ok_or(Error::<T>::Overflow)?;
        <MaxSocialTokenId<T>>::put(new_social_id);
        Ok(new_social_id)
    }

    /// Move `value` from the free balance from `who` to their reserved balance.
    ///
    /// Is a no-op if value to be reserved is zero.
    pub fn reserve(
        who: &T::AccountId,
        token_id: T::SocialTokenId,
        value: T::Balance,
    ) -> DispatchResult {
        if value.is_zero() {
            return Ok(());
        }

        Self::try_mutate_account(who, token_id, |account, _| -> DispatchResult {
            account.free = account
                .free
                .checked_sub(&value)
                .ok_or(Error::<T>::InsufficientBalance)?;
            account.reserved = account
                .reserved
                .checked_add(&value)
                .ok_or(Error::<T>::Overflow)?;
            Self::ensure_can_withdraw(
                &who,
                token_id,
                value.clone(),
                WithdrawReasons::RESERVE,
                account.free,
            )
        })?;

        Self::deposit_event(RawEvent::Reserved(who.clone(), token_id, value));
        Ok(())
    }

    /// Unreserve some funds, returning any amount that was unable to be unreserved.
    ///
    /// Is a no-op if the value to be unreserved is zero.
    pub fn unreserve(
        who: &T::AccountId,
        token_id: T::SocialTokenId,
        value: T::Balance,
    ) -> T::Balance {
		if value.is_zero() { return Zero::zero() }
		if Self::total_balance(&who, token_id).is_zero() { return value }

		let actual = match Self::mutate_account(who, token_id, |account| {
			let actual = cmp::min(account.reserved, value);
			account.reserved -= actual;
			// defensive only: this can never fail since total issuance which is at least free+reserved
			// fits into the same data type.
			account.free = account.free.saturating_add(actual);
			actual
		}) {
			Ok(x) => x,
			Err(_) => {
				// This should never happen since we don't alter the total amount in the account.
				// If it ever does, then we should fail gracefully though, indicating that nothing
				// could be done.
				return value
			}
		};

        Self::deposit_event(RawEvent::Unreserved(who.clone(), token_id, actual.clone()));
		value - actual
    }

    /// Slash from reserved balance, returning the negative imbalance created,
    /// and any amount that was unable to be slashed.
    ///
    /// Is a no-op if the value to be slashed is zero.
    pub fn slash_reserved(
        who: &T::AccountId,
        token_id: T::SocialTokenId,
        value: T::Balance,
    ) -> (NegativeImbalance<T>, T::Balance) {
        if value.is_zero() {
            return (NegativeImbalance::zero(), Zero::zero());
        }

        // NOTE: `mutate_account` may fail if it attempts to reduce the balance to the point that an
		//   account is attempted to be illegally destroyed.

		for attempt in 0..2 {
			match Self::mutate_account(who, token_id, |account| {
				let best_value = match attempt {
					0 => value,
					// If acting as a critical provider (i.e. first attempt failed), then ensure
					// slash leaves at least the ED.
					_ => value.min((account.free + account.reserved).saturating_sub(T::ExistentialDeposit::get())),
				};

				let actual = cmp::min(account.reserved, best_value);
				account.reserved -= actual;

				// underflow should never happen, but it if does, there's nothing to be done here.
				(NegativeImbalance::new(actual), value - actual)
			}) {
				Ok(r) => return r,
				Err(_) => (),
			}
		}
		// Should never get here as we ensure that ED is left in the second attempt.
		// In case we do, though, then we fail gracefully.
		(NegativeImbalance::zero(), value)
    }

    /// Similar to withdraw, only accepts a `PositiveImbalance` and returns nothing on success.
    pub fn settle(
        who: &T::AccountId,
        token_id: T::SocialTokenId,
        value: PositiveImbalance<T>,
        reasons: WithdrawReasons,
        liveness: ExistenceRequirement,
    ) -> result::Result<(), PositiveImbalance<T>> {
        let v = value.peek();
        match Self::withdraw(who, token_id, v, reasons, liveness) {
            Ok(opposite) => Ok(drop(value.offset(opposite))),
            _ => Err(value),
        }
    }

    /// Withdraw some free balance from an account, respecting existence requirements.
    ///
    /// Is a no-op if value to be withdrawn is zero.
    fn withdraw(
        who: &T::AccountId,
        token_id: T::SocialTokenId,
        value: T::Balance,
        reasons: WithdrawReasons,
        liveness: ExistenceRequirement,
    ) -> result::Result<NegativeImbalance<T>, DispatchError> {
        if value.is_zero() {
            return Ok(NegativeImbalance::zero());
        }

        Self::try_mutate_account(
            who,
            token_id,
            |account, _| -> Result<NegativeImbalance<T>, DispatchError> {
                let new_free_account = account
                    .free
                    .checked_sub(&value)
                    .ok_or(Error::<T>::InsufficientBalance)?;

                // bail if we need to keep the account alive and this would kill it.
                let ed = T::ExistentialDeposit::get();
                let would_be_dead = new_free_account + account.reserved < ed;
                let would_kill = would_be_dead && account.free + account.reserved >= ed;
                ensure!(liveness == AllowDeath || !would_kill, Error::<T>::KeepAlive);

                Self::ensure_can_withdraw(who, token_id, value, reasons, new_free_account)?;

                account.free = new_free_account;

                Ok(NegativeImbalance::new(value))
            },
        )
    }

    /// Deposit some `value` into the free balance of `who`, possibly creating a new account.
    ///
    /// This function is a no-op if:
    /// - the `value` to be deposited is zero; or
    /// - if the `value` to be deposited is less than the ED and the account does not yet exist; or
    /// - `value` is so large it would cause the balance of `who` to overflow.
    pub fn deposit_creating(
        who: &T::AccountId,
        token_id: T::SocialTokenId,
        value: T::Balance,
    ) -> PositiveImbalance<T> {
        if value.is_zero() {
            return PositiveImbalance::zero();
        }

        let r = Self::try_mutate_account(
            who,
            token_id,
            |account, is_new| -> Result<PositiveImbalance<T>, DispatchError> {

                let ed = T::ExistentialDeposit::get();
                ensure!(value >= ed || !is_new, Error::<T>::ExistentialDeposit);

                // defensive only: overflow should never happen, however in case it does, then this
                // operation is a no-op.
                account.free = match account.free.checked_add(&value) {
                    Some(x) => x,
                    None => return Ok(PositiveImbalance::zero()),
                };

                Ok(PositiveImbalance::new(value))
		}).unwrap_or_else(|_| PositiveImbalance::zero());

		r
    }

    // Burn funds from the total issuance, returning a positive imbalance for the amount burned.
    // Is a no-op if amount to be burned is zero.
    pub fn burn(token_id: T::SocialTokenId, mut amount: T::Balance) -> PositiveImbalance<T> {
        if amount.is_zero() {
            return PositiveImbalance::zero();
        }
        <TotalIssuance<T>>::mutate(token_id, |issued| {
            *issued = issued.checked_sub(&amount).unwrap_or_else(|| {
                amount = *issued;
                Zero::zero()
            });
        });
        PositiveImbalance::new(amount)
    }

    // Create new funds into the total issuance, returning a negative imbalance
    // for the amount issued.
    // Is a no-op if amount to be issued it zero.
    pub fn issue(token_id: T::SocialTokenId, mut amount: T::Balance) -> NegativeImbalance<T> {
        if amount.is_zero() {
            return NegativeImbalance::zero();
        }
        <TotalIssuance<T>>::mutate(token_id, |issued| {
            *issued = issued.checked_add(&amount).unwrap_or_else(|| {
                amount = <T as Config>::Balance::max_value() - *issued;
                <T as Config>::Balance::max_value()
            })
        });
        NegativeImbalance::new(amount)
    }

    /// Produce a pair of imbalances that cancel each other out exactly.
    ///
    /// This is just the same as burning and issuing the same amount and has no effect on the
    /// total issuance.
    pub fn pair(
        token_id: T::SocialTokenId,
        amount: T::Balance,
    ) -> (PositiveImbalance<T>, NegativeImbalance<T>) {
        (
            Self::burn(token_id, amount.clone()),
            Self::issue(token_id, amount),
        )
    }

    /// Mutate an account to some new value, or delete it entirely with `None`. Will enforce
    /// `ExistentialDeposit` law, annulling the account as needed. This will do nothing if the
    /// result of `f` is an `Err`.
    ///
    /// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
    /// when it is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    fn try_mutate_account<R, E: From<StoredMapError>>(
        who: &T::AccountId,
        token_id: T::SocialTokenId,
        f: impl FnOnce(&mut AccountData<T::Balance>, bool) -> Result<R, E>,
    ) -> Result<R, E> {
        Self::try_mutate_exists(&(token_id, who.clone()), |maybe_account| {
            let is_new = maybe_account.is_none();
            let mut account = maybe_account.take().unwrap_or_default();
            f(&mut account, is_new).map(move |result| {
                let maybe_endowed = if is_new { Some(account.free) } else { None };
                *maybe_account = Self::post_mutation(who, account);
                (maybe_endowed, result)
            })
        })
        .map(|(maybe_endowed, result)| {
            if let Some(endowed) = maybe_endowed {
                Self::deposit_event(RawEvent::Endowed(who.clone(), token_id, endowed));
            }
            result
        })
    }

    /// Mutate an account to some new value, or delete it entirely with `None`. Will enforce
    /// `ExistentialDeposit` law, annulling the account as needed.
    ///
    /// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
    /// when it is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    pub fn mutate_account<R>(
        who: &T::AccountId,
        token_id: T::SocialTokenId,
        f: impl FnOnce(&mut AccountData<T::Balance>) -> R,
    ) -> Result<R, StoredMapError> {
		Self::try_mutate_account(who, token_id, |a, _| -> Result<R, StoredMapError> { Ok(f(a)) })
    }

    /// Places the `free` and `reserved` parts of `new` into `account`. Also does any steps needed
    /// after mutating an account. This includes DustRemoval unbalancing, in the case than the `new`
    /// account's total balance is non-zero but below ED.
    ///
    /// Returns the final free balance, iff the account was previously of total balance zero, known
    /// as its "endowment".
    fn post_mutation(
        _who: &T::AccountId,
        new: AccountData<T::Balance>,
    ) -> Option<AccountData<T::Balance>> {
        let total = new.total();
        if total < T::ExistentialDeposit::get() {
            // TODO:
            /*
            if !total.is_zero() {
                T::DustRemoval::on_unbalanced(NegativeImbalance::new(total));
                Self::deposit_event(RawEvent::DustLost(who.clone(), total));
            }
            */
            None
        } else {
            Some(new)
        }
    }

    // Ensure that an account can withdraw from their free balance given any existing withdrawal
    // restrictions like locks and vesting balance.
    // Is a no-op if amount to be withdrawn is zero.
    //
    // # <weight>
    // Despite iterating over a list of locks, they are limited by the number of
    // lock IDs, which means the number of runtime modules that intend to use and create locks.
    // # </weight>
    fn ensure_can_withdraw(
        who: &T::AccountId,
        token_id: T::SocialTokenId,
        amount: T::Balance,
        reasons: WithdrawReasons,
        new_balance: T::Balance,
    ) -> DispatchResult {
        if amount.is_zero() {
            return Ok(());
        }
        let min_balance = Self::account(token_id, who).frozen(reasons.into());
        ensure!(
            new_balance >= min_balance,
            Error::<T>::LiquidityRestrictions
        );
        Ok(())
    }

    /// Get both the free and reserved balances of an account.
    fn account(token_id: T::SocialTokenId, who: &T::AccountId) -> AccountData<T::Balance> {
        Self::get(&(token_id, who.clone()))
    }

    pub fn minimum_balance() -> T::Balance {
        T::ExistentialDeposit::get()
    }

    /// An account is being created.
    pub fn on_created_account(who: (T::SocialTokenId, T::AccountId)) {
        <T as Config>::OnNewAccount::on_new_account(&who);
        Self::deposit_event(RawEvent::NewAccount(who.1, who.0));
    }
}

// wrapping these imbalances in a private module is necessary to ensure absolute privacy
// of the inner member.
mod imbalances {
    use super::{result, Imbalance, Saturating, Config, TryDrop, Zero};
    use sp_std::mem;

    /// Opaque, move-only struct with private fields that serves as a token denoting that
    /// funds have been created without any equal and opposite accounting.
    #[must_use]
    #[derive(Clone)]
    pub struct PositiveImbalance<T: Config>(T::Balance);

    impl<T: Config> PositiveImbalance<T> {
        /// Create a new positive imbalance from a balance.
        pub fn new(amount: T::Balance) -> Self {
            PositiveImbalance(amount)
        }
    }

    /// Opaque, move-only struct with private fields that serves as a token denoting that
    /// funds have been destroyed without any equal and opposite accounting.
    #[must_use]
    #[derive(Clone)]
    pub struct NegativeImbalance<T: Config>(T::Balance);

    impl<T: Config> NegativeImbalance<T> {
        /// Create a new negative imbalance from a balance.
        pub fn new(amount: T::Balance) -> Self {
            NegativeImbalance(amount)
        }
    }

    impl<T: Config> TryDrop for PositiveImbalance<T> {
        fn try_drop(self) -> result::Result<(), Self> {
            self.drop_zero()
        }
    }

    impl<T: Config> Imbalance<T::Balance> for PositiveImbalance<T> {
        type Opposite = NegativeImbalance<T>;

        fn zero() -> Self {
            Self(Zero::zero())
        }
        fn drop_zero(self) -> result::Result<(), Self> {
            if self.0.is_zero() {
                Ok(())
            } else {
                Err(self)
            }
        }
        fn split(self, amount: T::Balance) -> (Self, Self) {
            let first = self.0.min(amount);
            let second = self.0 - first;

            mem::forget(self);
            (Self(first), Self(second))
        }
        fn merge(mut self, other: Self) -> Self {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);

            self
        }
        fn subsume(&mut self, other: Self) {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);
        }
        fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
            let (a, b) = (self.0, other.0);
            mem::forget((self, other));

            if a >= b {
                Ok(Self(a - b))
            } else {
                Err(NegativeImbalance::new(b - a))
            }
        }
        fn peek(&self) -> T::Balance {
            self.0.clone()
        }
    }

    impl<T: Config> TryDrop for NegativeImbalance<T> {
        fn try_drop(self) -> result::Result<(), Self> {
            self.drop_zero()
        }
    }

    impl<T: Config> Imbalance<T::Balance> for NegativeImbalance<T> {
        type Opposite = PositiveImbalance<T>;

        fn zero() -> Self {
            Self(Zero::zero())
        }
        fn drop_zero(self) -> result::Result<(), Self> {
            if self.0.is_zero() {
                Ok(())
            } else {
                Err(self)
            }
        }
        fn split(self, amount: T::Balance) -> (Self, Self) {
            let first = self.0.min(amount);
            let second = self.0 - first;

            mem::forget(self);
            (Self(first), Self(second))
        }
        fn merge(mut self, other: Self) -> Self {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);

            self
        }
        fn subsume(&mut self, other: Self) {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);
        }
        fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
            let (a, b) = (self.0, other.0);
            mem::forget((self, other));

            if a >= b {
                Ok(Self(a - b))
            } else {
                Err(PositiveImbalance::new(b - a))
            }
        }
        fn peek(&self) -> T::Balance {
            self.0.clone()
        }
    }
}

/// Information of an account.
#[derive(Clone, Eq, PartialEq, Default, RuntimeDebug, Encode, Decode)]
pub struct AccountInfo<Index, AccountData> {
    /// The number of transactions this account has sent.
    pub nonce: Index,
    /// The number of other modules that currently depend on this account's existence. The account
    /// cannot be reaped until this is zero.
    pub refcount: RefCount,
    /// The additional data that belongs to this account. Used to store the balance(s) in a lot of
    /// chains.
    pub data: AccountData,
}

// Implement StoredMap for a simple single-item, kill-account-on-remove system. This works fine for
// storing a single item which is required to not be empty/default for the account to exist.
// Anything more complex will need more sophisticated logic.
impl<T: Config> StoredMap<(T::SocialTokenId, T::AccountId), AccountData<T::Balance>> for Module<T> {
    fn get(k: &(T::SocialTokenId, T::AccountId)) -> AccountData<T::Balance> {
        SystemAccount::<T>::get(k).data
    }
    fn insert(
        k: &(T::SocialTokenId, T::AccountId),
        data: AccountData<T::Balance>
    ) -> Result<(), StoredMapError> {
        let existed = SystemAccount::<T>::contains_key(k);
        let r = SystemAccount::<T>::mutate(k, |a| a.data = data);
        if !existed {
            Self::on_created_account(k.clone());
        }
        Ok(r)
    }
    fn remove(_k: &(T::SocialTokenId, T::AccountId)) -> Result<(), StoredMapError> {
        // TODO:
        //Self::kill_account(k)
        Ok(())
    }
    fn mutate<R>(
        k: &(T::SocialTokenId, T::AccountId),
        f: impl FnOnce(&mut AccountData<T::Balance>) -> R,
    ) -> Result<R, StoredMapError> {
        let existed = SystemAccount::<T>::contains_key(k);
        let r = SystemAccount::<T>::mutate(k, |a| f(&mut a.data));
        if !existed {
            Self::on_created_account(k.clone());
        }
        Ok(r)
    }
    fn mutate_exists<R>(
        k: &(T::SocialTokenId, T::AccountId),
        f: impl FnOnce(&mut Option<AccountData<T::Balance>>) -> R,
    ) -> Result<R, StoredMapError> {
        Self::try_mutate_exists(k, |x| -> Result<R, StoredMapError> { Ok(f(x)) })
    }
    fn try_mutate_exists<R, E: From<StoredMapError>>(
        k: &(T::SocialTokenId, T::AccountId),
        f: impl FnOnce(&mut Option<AccountData<T::Balance>>) -> Result<R, E>,
    ) -> Result<R, E> {
        SystemAccount::<T>::try_mutate_exists(k, |maybe_value| {
            let existed = maybe_value.is_some();
            let (maybe_prefix, mut maybe_data) = split_inner(maybe_value.take(), |account| {
                ((account.nonce, account.refcount), account.data)
            });
            f(&mut maybe_data).map(|result| {
                *maybe_value = maybe_data.map(|data| {
                    let (nonce, refcount) = maybe_prefix.unwrap_or_default();
                    AccountInfo {
                        nonce,
                        refcount,
                        data,
                    }
                });
                (existed, maybe_value.is_some(), result)
            })
        })
        .map(|(existed, exists, v)| {
            if !existed && exists {
                Self::on_created_account(k.clone());
            } else if existed && !exists {
                // TODO:
                //Self::on_killed_account(k.clone());
            }
            v
        })
    }
}

pub trait Fungible<SocialTokenId, AccountId> {
    type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

    fn total_supply(token_id: &SocialTokenId) -> Self::Balance;

    fn balances(token_id: &SocialTokenId, who: &AccountId) -> Self::Balance;

    fn allowances(
        token_id: &SocialTokenId,
        owner: &AccountId,
        spender: &AccountId,
    ) -> Self::Balance;

    fn transfer(
        token_id: &SocialTokenId,
        from: &AccountId,
        to: &AccountId,
        value: Self::Balance,
    ) -> DispatchResult;

    fn transfer_from(
        token_id: &SocialTokenId,
        from: &AccountId,
        operator: &AccountId,
        to: &AccountId,
        value: Self::Balance,
    ) -> DispatchResult;
}

pub trait IssueAndBurn<SocialTokenId, AccountId>: Fungible<SocialTokenId, AccountId> {
    fn exists(asset_id: &SocialTokenId) -> bool;

    fn create_new_asset(dossier: TokenDossier) -> Result<SocialTokenId, DispatchError>;

    fn issue(asset_id: &SocialTokenId, who: &AccountId, value: Self::Balance) -> DispatchResult;

    fn burn(asset_id: &SocialTokenId, who: &AccountId, value: Self::Balance) -> DispatchResult;
}

impl<T: Config> Fungible<T::SocialTokenId, T::AccountId> for Module<T> {
    type Balance = T::Balance;

    fn total_supply(token_id: &T::SocialTokenId) -> Self::Balance {
        <TotalIssuance<T>>::get(token_id)
    }

    fn balances(token_id: &T::SocialTokenId, who: &T::AccountId) -> Self::Balance {
        Self::balance(who.clone(), *token_id)
    }

    fn allowances(
        token_id: &T::SocialTokenId,
        owner: &T::AccountId,
        spender: &T::AccountId,
    ) -> Self::Balance {
        Self::allowances(token_id, (owner, spender))
    }

    fn transfer(
        token_id: &T::SocialTokenId,
        from: &T::AccountId,
        to: &T::AccountId,
        value: Self::Balance,
    ) -> DispatchResult {
        Self::do_transfer(
            &from,
            &to,
            *token_id,
            value,
            ExistenceRequirement::AllowDeath,
        )
    }

    fn transfer_from(
        token_id: &T::SocialTokenId,
        from: &T::AccountId,
        operator: &T::AccountId,
        to: &T::AccountId,
        value: Self::Balance,
    ) -> DispatchResult {
        let new_allowance = Self::allowances(token_id, (from, operator))
            .checked_sub(&value)
            .ok_or(Error::<T>::NotAllowed)?;

        if from != to {
            Self::do_transfer(
                &from,
                &to,
                *token_id,
                value,
                ExistenceRequirement::AllowDeath,
            )?;
        }

        <Allowances<T>>::mutate(token_id, (from, operator), |approved_balance| {
            *approved_balance = new_allowance;
        });

        Ok(())
    }
}

impl<T: Config> IssueAndBurn<T::SocialTokenId, T::AccountId> for Module<T> {
    fn exists(asset_id: &T::SocialTokenId) -> bool {
        Self::token_info(asset_id).is_some()
    }

    fn create_new_asset(dossier: TokenDossier) -> Result<T::SocialTokenId, DispatchError> {
        let id = Self::create_new_social_token()?;
        <TokenInfo<T>>::insert(id, dossier);
        Ok(id)
    }

    fn issue(
        token_id: &T::SocialTokenId,
        who: &T::AccountId,
        value: Self::Balance,
    ) -> DispatchResult {
        ensure!(Self::exists(token_id), Error::<T>::InvalidSocialTokenId);

        Self::mint_social_token(who.clone(), *token_id, value);
        let _ = Self::issue(*token_id, value);
        Self::deposit_event(RawEvent::Issued(*token_id, who.clone(), value));

        Ok(())
    }

    fn burn(
        token_id: &T::SocialTokenId,
        who: &T::AccountId,
        value: Self::Balance,
    ) -> DispatchResult {
        ensure!(Self::exists(token_id), Error::<T>::InvalidSocialTokenId);

        Self::burn_social_token(who.clone(), *token_id, value);
        let _ = Self::burn(*token_id, value);

        Ok(())
    }
}
