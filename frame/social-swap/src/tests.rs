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
		let intial_supply = 20000;
		pallet_assets::Module::<Test>::issue(&asset_id, &account_id, intial_supply);
		assert_eq!(
			pallet_assets::Module::<Test>::total_supply(lp_token),
			0
		);
		assert_eq!(
			SocialSwap::add_liquidity(
				Origin::signed(account_id),
				exchange_id,
				ETH_RESERVE,
				0,
				HAY_RESERVE,
				1
			),
			Ok(())
		);
		assert_eq!(Balances::total_balance(&SocialSwap::account_id()),
				   INITIAL_BALANCE + ETH_RESERVE);
		assert_eq!(
			pallet_assets::Module::<Test>::total_supply(lp_token),
			ETH_RESERVE
		);
		assert_eq!(
			pallet_assets::Module::<Test>::balance(asset_id, SocialSwap::account_id()),
			HAY_RESERVE
		);

		let mut exchange = Exchanges::<Test>::get(exchange_id).unwrap();
		exchange.native_token_amount = ETH_RESERVE;
		exchange.trade_token_amount = 0;
		<Exchanges<Test>>::mutate(&exchange_id, |e| *e = Some(exchange));


		assert_eq!(
			SocialSwap::add_liquidity(
				Origin::signed(account_id),
				exchange_id,
				ETH_ADDED,
				1,
				15*10^18,
				1
			),
			Ok(())
		);
		assert_eq!(
			pallet_assets::Module::<Test>::total_supply(lp_token),
			ETH_RESERVE + ETH_ADDED
		);
		assert_eq!(Balances::total_balance(&SocialSwap::account_id()),
				   INITIAL_BALANCE + ETH_RESERVE + ETH_ADDED);
		assert_eq!(
			pallet_assets::Module::<Test>::balance(asset_id, SocialSwap::account_id()),
			HAY_RESERVE + 1
		);
	});
}

#[test]
fn test_remove_liquidity_with_wrong_deadline_should_not_work() {
	new_test_ext().execute_with(|| {
		let liquidity_burned = 1*10^18;
		let (exchange_id, _, _) = create_exchange_test();
		assert_noop!(
			SocialSwap::remove_liquidity(
				Origin::signed(1),
				exchange_id,
				liquidity_burned,
				1,
				1,
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
				1,
				1,
				1,
				1
			),
			Error::<Test>::TradeTokenNotExists
		);

	});
}

#[test]
fn test_remove_liquidity_with_not_enough_tokens_should_not_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, asset_id, _) = create_exchange_test();
		let liquidity_burned = 1*10^18;
		pallet_assets::Module::<Test>::issue(&asset_id, &OWNER, 100);
		let account_id: u64 = 1;
		assert_noop!(
			SocialSwap::remove_liquidity(
				Origin::signed(account_id),
				exchange_id,
				0,
				1,
				1,
				1
			),
			Error::<Test>::NotQualifiedBurn
		);

		assert_noop!(
			SocialSwap::remove_liquidity(
				Origin::signed(account_id),
				exchange_id,
				liquidity_burned,
				0,
				1,
				1
			),
			Error::<Test>::NotQualifiedBurn
		);

		assert_noop!(
			SocialSwap::remove_liquidity(
				Origin::signed(account_id),
				exchange_id,
				liquidity_burned,
				1,
				0,
				1
			),
			Error::<Test>::NotQualifiedBurn
		);

	});
}

#[test]
fn test_remove_liquidity_with_not_enough_liquidity_should_not_work() {
	new_test_ext().execute_with(|| {
		let (exchange_id, asset_id, _) = create_exchange_test();
		pallet_assets::Module::<Test>::issue(&asset_id, &OWNER, 100);
		let account_id: u64 = 1;
		let liquidity_burned = 1*10^18;
		assert_noop!(
			SocialSwap::remove_liquidity(
				Origin::signed(account_id),
				exchange_id,
				liquidity_burned + 1,
				1,
				1,
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
		let intial_supply = 20000;
		pallet_assets::Module::<Test>::issue(&asset_id, &account_id, intial_supply);
		assert_eq!(
			SocialSwap::add_liquidity(
				Origin::signed(account_id),
				exchange_id,
				ETH_RESERVE,
				0,
				HAY_RESERVE,
				1
			),
			Ok(())
		);
		let mut exchange = Exchanges::<Test>::get(exchange_id).unwrap();
		exchange.native_token_amount = ETH_RESERVE;
		exchange.trade_token_amount = 0;
		<Exchanges<Test>>::mutate(&exchange_id, |e| *e = Some(exchange));


		assert_eq!(
			SocialSwap::add_liquidity(
				Origin::signed(account_id),
				exchange_id,
				ETH_ADDED,
				1,
				15*10^18,
				1
			),
			Ok(())
		);

		assert_eq!(
			pallet_assets::Module::<Test>::total_supply(lp_token),
			ETH_RESERVE + ETH_ADDED
		);

		assert_eq!(Balances::total_balance(&SocialSwap::account_id()),
				   INITIAL_BALANCE + ETH_RESERVE + ETH_ADDED);

		let mut exchange = Exchanges::<Test>::get(exchange_id).unwrap();
		exchange.native_token_amount = 1*10^24;
		exchange.trade_token_amount = 1*10^18;
		<Exchanges<Test>>::mutate(&exchange_id, |e| *e = Some(exchange));

		assert_eq!(
			SocialSwap::remove_liquidity(
				Origin::signed(account_id),
				exchange_id,
				ETH_RESERVE,
				1,
				1,
				1
			),
			Ok(())
		);

		assert_eq!(
			SocialSwap::remove_liquidity(
				Origin::signed(account_id),
				exchange_id,
				ETH_ADDED - 1*10^18,
				1,
				1,
				1
			),
			Ok(())
		);

		assert_eq!(
			pallet_assets::Module::<Test>::total_supply(lp_token),
			0
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
#[ignore]
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
#[ignore]
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
#[ignore]
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
#[ignore]
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
