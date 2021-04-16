use crate::{mock::*, Error};
use frame_support::assert_noop;
use pallet_assets::*;
use super::*;

fn create_exchange_test() -> (u64, u32, u32) {
	let max_zombies:u32 = 3;
	let min_balance:u32 = 10;

	assert_eq!(
		SocialSwap::create_exchange(
			Origin::signed(1),
			ASSET_ID,
			max_zombies,
			min_balance
		),
		Ok(())
	);
	let exchange_id = TradeTokenToExchange::<Test>::get(ASSET_ID).unwrap();
	let exchange = Exchanges::<Test>::get(exchange_id).unwrap();
	(exchange_id, ASSET_ID, exchange.lp_token)
}

#[test]
fn test_create_exchange_wrong_token_id_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			SocialSwap::create_exchange(
				Origin::signed(1),
				11,
				MAX_ZOMBIES,
				MIN_BALANCE as u32
			),
			Error::<Test>::TradeTokenNotExists
		);
	});
}

#[test]
fn test_create_exchange_token_id_without_exchange_should_not_work() {
	new_test_ext().execute_with(|| {

		TradeTokenToExchange::<Test>::insert(ASSET_ID, 1u64);
		assert_noop!(
			SocialSwap::create_exchange(
				Origin::signed(1),
				ASSET_ID,
				MAX_ZOMBIES,
				MIN_BALANCE as u32
			),
			Error::<Test>::ExchangeExists
		);
	});
}

#[test]
fn test_create_exchange_token_should_work() {
	new_test_ext().execute_with(|| {

		assert_eq!(
			SocialSwap::create_exchange(
				Origin::signed(1),
				ASSET_ID,
				MAX_ZOMBIES,
				MIN_BALANCE as u32
			),
			Ok(())
		);
		assert_eq!(TradeTokenToExchange::<Test>::contains_key(ASSET_ID), true);
		let exchange_id = TradeTokenToExchange::<Test>::get(ASSET_ID).unwrap();
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
		let (exchange_id, asset_id, _) = create_exchange_test();
		let account_id: u64 = 1;
		pallet_assets::Module::<Test>::issue(&asset_id, &OWNER, 100);
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
		let (exchange_id, asset_id, lp_token) = create_exchange_test();
		let account_id: u64 = 2;
		let mut exchange = Exchanges::<Test>::get(exchange_id).unwrap();
		exchange.native_token_amount = 40000;
		exchange.trade_token_amount = 40000;
		<Exchanges<Test>>::mutate(&exchange_id, |e| *e = Some(exchange));
		pallet_assets::Module::<Test>::issue(&asset_id, &account_id, 20000);
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
		let (exchange_id, asset_id, _) = create_exchange_test();
		pallet_assets::Module::<Test>::issue(&asset_id, &OWNER, 100);
		let account_id: u64 = 1;
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
		let (exchange_id, asset_id, _) = create_exchange_test();
		pallet_assets::Module::<Test>::issue(&asset_id, &OWNER, 100);
		let account_id: u64 = 1;
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
		let (exchange_id, asset_id, lp_token) = create_exchange_test();
		let account_id: u64 = 2;
		pallet_assets::Module::<Test>::issue(&asset_id, &account_id, 101);
		pallet_assets::Module::<Test>::issue(&asset_id, &SocialSwap::account_id(), 20000);
		pallet_assets::Module::<Test>::issue(&lp_token, &account_id, 100);
		let mut exchange = Exchanges::<Test>::get(exchange_id).unwrap();
		exchange.native_token_amount = 400;
		exchange.trade_token_amount = 400;
		<Exchanges<Test>>::mutate(&exchange_id, |e| *e = Some(exchange));
		assert_eq!(
			SocialSwap::remove_liquidity(
				Origin::signed(account_id),
				exchange_id,
				10,
				1,
				1,
				1
			),
			Ok(())
		);

	});
}

#[test]
fn test_native_to_trade_token_input_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			SocialSwap::native_to_trade_token_input(
				Origin::signed(1),
				1,
				1000,
				12,
				10,
				1
			),
			Error::<Test>::ExchangeNotExists
		);
	});
}

#[test]
fn test_native_to_trade_token_input_should_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, asset_id, lp_token) = create_exchange_test();
		let account_id: u64 = 1;
		assert_eq!(
			SocialSwap::native_to_trade_token_input(
				Origin::signed(account_id),
				exchange_id,
				1000,
				10,
				1,
				2
			),
			Ok(())
		);
	});
}

#[test]
fn test_trade_to_native_token_input_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			SocialSwap::trade_to_native_token_input(
				Origin::signed(1),
				1,
				1000,
				12,
				10,
				1
			),
			Error::<Test>::ExchangeNotExists
		);
	});
}

#[test]
fn test_trade_to_native_token_input_should_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, asset_id, lp_token) = create_exchange_test();
		let account_id: u64 = 1;
		assert_eq!(
			SocialSwap::trade_to_native_token_input(
				Origin::signed(account_id),
				exchange_id,
				1000,
				10,
				1,
				2
			),
			Ok(())
		);
	});
}

#[test]
fn test_native_to_trade_token_output_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			SocialSwap::native_to_trade_token_output(
				Origin::signed(1),
				1,
				1000,
				10,
				1
			),
			Error::<Test>::ExchangeNotExists
		);
	});
}

#[test]
fn test_native_to_trade_token_output_should_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, asset_id, lp_token) = create_exchange_test();
		let account_id: u64 = 1;
		let mut exchange = Exchanges::<Test>::get(exchange_id).unwrap();
		exchange.native_token_amount = 40000;
		exchange.trade_token_amount = 40000;
		<Exchanges<Test>>::mutate(&exchange_id, |e| *e = Some(exchange));
		assert_eq!(
			SocialSwap::native_to_trade_token_output(
				Origin::signed(account_id),
				exchange_id,
				1000,
				1,
				2
			),
			Ok(())
		);
	});
}

#[test]
fn test_trade_to_native_token_output_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			SocialSwap::trade_to_native_token_output(
				Origin::signed(1),
				1,
				1000,
				10,
				1
			),
			Error::<Test>::ExchangeNotExists
		);
	});
}

#[test]
fn test_trade_to_native_token_output_should_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, asset_id, lp_token) = create_exchange_test();
		let account_id: u64 = 1;
		let mut exchange = Exchanges::<Test>::get(exchange_id).unwrap();
		exchange.native_token_amount = 40000;
		exchange.trade_token_amount = 40000;
		<Exchanges<Test>>::mutate(&exchange_id, |e| *e = Some(exchange));
		assert_eq!(
			SocialSwap::trade_to_native_token_output(
				Origin::signed(account_id),
				exchange_id,
				1000,
				1,
				2
			),
			Ok(())
		);
	});
}

