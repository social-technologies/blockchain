use crate::{mock::*, Error};
use frame_support::assert_noop;
use super::*;
use pallet_social_tokens::*;

fn create_exchange_test() -> (u64, u32, u32) {
	let social_token_id: u32 = SocialSwap::create_lp_token(1).unwrap();
	assert_eq!(
		SocialSwap::create_exchange(
			Origin::signed(1),
			social_token_id
		),
		Ok(())
	);
	let exchange_id = TradeTokenToExchange::<Test>::get(social_token_id).unwrap();
	let exchange = Exchanges::<Test>::get(exchange_id).unwrap();
	(exchange_id, social_token_id, exchange.lp_token)
}

#[test]
fn test_create_exchange_wrong_token_id_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			SocialSwap::create_exchange(
				Origin::signed(1),
				1
			),
			Error::<Test>::TradeTokenNotExists
		);
	});
}

#[test]
fn test_create_exchange_token_id_without_exchange_should_not_work() {
	new_test_ext().execute_with(|| {
		let social_token_id: u32 = SocialSwap::create_lp_token(1).unwrap();
		TradeTokenToExchange::<Test>::insert(social_token_id, 1u64);
		assert_noop!(
			SocialSwap::create_exchange(
				Origin::signed(1),
				social_token_id
			),
			Error::<Test>::ExchangeExists
		);
	});
}

#[test]
fn test_create_exchange_token_should_work() {
	new_test_ext().execute_with(|| {
		let social_token_id: u32 = SocialSwap::create_lp_token(1).unwrap();
		assert_eq!(
			SocialSwap::create_exchange(
				Origin::signed(1),
				social_token_id
			),
			Ok(())
		);
		assert_eq!(TradeTokenToExchange::<Test>::contains_key(social_token_id), true);
		let exchange_id = TradeTokenToExchange::<Test>::get(social_token_id).unwrap();
		assert_eq!(Exchanges::<Test>::contains_key(exchange_id), true);
	});
}

#[test]
fn test_add_liquidity_with_wrong_deadline_should_not_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, _, _) = create_exchange_test();
		assert_noop!(
			SocialSwap::add_liquidity(
				Origin::signed(1),
				exchange_id,
				20000,
				1000,
				10,
				0
			),
			Error::<Test>::TooLate
		);

	});
}

#[test]
fn test_add_liquidity_with_non_existing_token_should_not_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, _, _) = create_exchange_test();
		assert_noop!(
			SocialSwap::add_liquidity(
				Origin::signed(1),
				exchange_id,
				20000,
				1000,
				10,
				1
			),
			Error::<Test>::TradeTokenNotExists
		);

	});
}

#[test]
fn test_add_liquidity_with_zero_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, social_token_id, _) = create_exchange_test();
		let account_id: u64 = 1;
		pallet_social_tokens::Module::<Test>::issue(social_token_id, 100);
		let data = AccountData{free: 100_000_0, fee_frozen: 0, misc_frozen: 0, reserved: 0};
		let account_store = AccountInfo{data, nonce: 1, refcount: 0};
		pallet_social_tokens::SystemAccount::<Test>::insert((social_token_id, account_id), account_store);
		assert_eq!(
			SocialSwap::add_liquidity(
				Origin::signed(account_id),
				exchange_id,
				20000,
				1000,
				10,
				1
			),
			Ok(())
		);

	});
}


#[test]
fn test_add_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, social_token_id, lp_token) = create_exchange_test();
		let account_id: u64 = 1;
		pallet_social_tokens::Module::<Test>::issue(social_token_id, 100);
		pallet_social_tokens::Module::<Test>::issue(lp_token, 100);
		let data = AccountData{free: 100_000_0, fee_frozen: 0, misc_frozen: 0, reserved: 0};
		let account_store = AccountInfo{data, nonce: 1, refcount: 0};
		pallet_social_tokens::SystemAccount::<Test>::insert((social_token_id, account_id), account_store);
		let mut exchange = Exchanges::<Test>::get(exchange_id).unwrap();
		SocialSwap::input_liquidity(&account_id.clone(), 100, &social_token_id, 100, &mut exchange);
		<Exchanges<Test>>::mutate(&exchange_id, |e| *e = Some(exchange));
		assert_eq!(
			SocialSwap::add_liquidity(
				Origin::signed(account_id),
				exchange_id,
				2000,
				2000,
				2001,
				1
			),
			Ok(())
		);

	});
}

#[test]
fn test_remove_liquidity_with_wrong_deadline_should_not_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, _, _) = create_exchange_test();
		assert_noop!(
			SocialSwap::remove_liquidity(
				Origin::signed(1),
				exchange_id,
				20000,
				1000,
				10,
				0
			),
			Error::<Test>::TooLate
		);

	});
}

#[test]
fn test_remove_liquidity_with_non_existing_token_should_not_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, _, _) = create_exchange_test();
		assert_noop!(
			SocialSwap::remove_liquidity(
				Origin::signed(1),
				exchange_id,
				20000,
				1000,
				10,
				1
			),
			Error::<Test>::TradeTokenNotExists
		);

	});
}

#[test]
fn test_remove_liquidity_with_non_enough_burn_should_not_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, social_token_id, _) = create_exchange_test();
		let account_id: u64 = 1;
		pallet_social_tokens::Module::<Test>::issue(social_token_id, 100);
		assert_noop!(
			SocialSwap::remove_liquidity(
				Origin::signed(account_id),
				exchange_id,
				20000,
				0,
				0,
				1
			),
			Error::<Test>::NotQualifiedBurn
		);

	});
}

#[test]
fn test_remove_liquidity_with_non_enough_liquidity_should_not_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, social_token_id, _) = create_exchange_test();
		let account_id: u64 = 1;
		pallet_social_tokens::Module::<Test>::issue(social_token_id, 100);
		assert_noop!(
			SocialSwap::remove_liquidity(
				Origin::signed(account_id),
				exchange_id,
				20000,
				10,
				10,
				1
			),
			Error::<Test>::NotEnoughLiquidity
		);

	});
}

#[test]
fn test_remove_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, social_token_id, lp_token) = create_exchange_test();
		let account_id: u64 = 1;
		pallet_social_tokens::Module::<Test>::issue(social_token_id, 100);
		pallet_social_tokens::Module::<Test>::issue(lp_token, 1000);
		let data = AccountData{free: 500_000_0, fee_frozen: 10, misc_frozen: 10, reserved: 10};
		let account_store = AccountInfo{data, nonce: 1, refcount: 0};
		pallet_social_tokens::SystemAccount::<Test>::insert((social_token_id, account_id), account_store.clone());
		pallet_social_tokens::SystemAccount::<Test>::insert((social_token_id, SocialSwap::account_id()), account_store.clone());
		pallet_social_tokens::SystemAccount::<Test>::insert((lp_token, account_id), account_store);
		let mut exchange = Exchanges::<Test>::get(exchange_id).unwrap();
		SocialSwap::input_liquidity(&account_id.clone(), 1000, &social_token_id, 300, &mut exchange);
		<Exchanges<Test>>::mutate(&exchange_id, |e| *e = Some(exchange));
		assert_eq!(
			SocialSwap::remove_liquidity(
				Origin::signed(account_id),
				exchange_id,
				1000,
				12,
				10,
				1
			),
			Ok(())
		);

	});
}
