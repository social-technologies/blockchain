use crate::{mock::*, Error};
use super::*;
use frame_support::assert_noop;
use codec::Encode;

#[test]
fn test_mint_should_not_work() {
	new_test_ext().execute_with(|| {
		<Reserve0<Test>>::put(INITIAL_BALANCE);
		assert_noop!(
			SocialSwap2::mint(
				Origin::signed(ACCOUNT1),
				ACCOUNT3
			),
			Error::<Test>::NotEnoughLiquidity
		);
	});
}

#[test]
fn test_mint_should_work() {
	new_test_ext().execute_with(|| {

		SocialSwap2::initialize(Origin::root(), FEE_TO, ADDRESS0, TREASURY, TOKEN0, TOKEN1);
		let token_0_amount: u128  = 1_000_000_000_000_000_000;
		let token_1_amount: u128  = 4_000_000_000_000_000_000;
		pallet_assets::Module::<Test>::transfer(&ASSET_ID, &ACCOUNT1, &TOKEN0, token_0_amount);
		pallet_assets::Module::<Test>::transfer(&ASSET_ID, &ACCOUNT1, &TOKEN1, token_1_amount);
		let expected_liquidity: u128 = 2_000_000_000_000_000_000u128;

		assert_eq!(
			pallet_assets::Module::<Test>::total_supply(ASSET_ID),
			0
		);
		assert_eq!(
			SocialSwap2::mint(
				Origin::signed(ACCOUNT1),
				ACCOUNT2
			),
			Ok(())
		);

		assert_eq!(
			pallet_assets::Module::<Test>::total_supply(ASSET_ID),
			expected_liquidity
		);

		assert_eq!(
			pallet_assets::Module::<Test>::balance(ASSET_ID, ACCOUNT2),
			expected_liquidity - MINIMUM_LIQUIDITY as u128
		);

		assert_eq!(
			pallet_assets::Module::<Test>::balance(ASSET_ID, TOKEN0),
			token_0_amount
		);

		assert_eq!(
			pallet_assets::Module::<Test>::balance(ASSET_ID, TOKEN1),
			token_1_amount
		);

		assert_eq!(
			SocialSwap2::reserve0(),
			token_0_amount
		);

		assert_eq!(
			SocialSwap2::reserve1(),
			token_1_amount
		);
	});
}

#[test]
fn test_burn_should_not_work() {
	new_test_ext().execute_with(|| {

		assert_ne!(SocialSwap2::mint(
				Origin::signed(ACCOUNT1),
				ACCOUNT3
			),Ok(()));

	});
}

#[test]
fn test_burn_should_work() {
	new_test_ext().execute_with(|| {
		let token_0_amount: u128  = 3_000_000_000_000_000_000;
		let token_1_amount: u128  = 3_000_000_000_000_000_000;
		add_liquidity(token_0_amount, token_1_amount);
		let expected_liquidity: u128 = 3_000_000_000_000_000_000u128;

		assert_noop!(
			SocialSwap2::burn(
				Origin::signed(ACCOUNT1),
				ACCOUNT2
			),
			Error::<Test>::InsufficientLiquidityBurned
		);

		pallet_balances::Module::<Test>::deposit_creating(&TREASURY, expected_liquidity - MINIMUM_LIQUIDITY as u128);

		assert_eq!(
			pallet_assets::Module::<Test>::total_supply(ASSET_ID),
			expected_liquidity
		);


		assert_eq!(
			SocialSwap2::burn(
				Origin::signed(ACCOUNT1),
				ACCOUNT2
			),
			Ok(())
		);

		assert_eq!(
			SocialSwap2::reserve0(),
			MINIMUM_LIQUIDITY as u128
		);

		assert_eq!(
			SocialSwap2::reserve1(),
			MINIMUM_LIQUIDITY as u128
		);

		assert_eq!(
			pallet_assets::Module::<Test>::total_supply(ASSET_ID),
			MINIMUM_LIQUIDITY as u128
		);

		assert_eq!(
			pallet_assets::Module::<Test>::balance(ASSET_ID, TOKEN0),
			MINIMUM_LIQUIDITY as u128
		);

		assert_eq!(
			pallet_assets::Module::<Test>::balance(ASSET_ID, TOKEN1),
			MINIMUM_LIQUIDITY as u128
		);

		assert_eq!(
			pallet_assets::Module::<Test>::balance(ASSET_ID, ACCOUNT2),
			token_0_amount + token_1_amount - 2000u128
		);

	});
}

#[test]
fn test_swap_should_not_work() {
	new_test_ext().execute_with(|| {

		assert_noop!(SocialSwap2::swap(
			Origin::signed(ACCOUNT1),
			0,
			0,
			ACCOUNT3,
			"0x".encode()
		),
		Error::<Test>::InsufficientOutputAmount);

		assert_noop!(SocialSwap2::swap(
			Origin::signed(ACCOUNT1),
			1,
			2,
			ACCOUNT3,
			"0x".encode()
		),
		Error::<Test>::InsufficientLiquidity);

		<Reserve0<Test>>::put(2);
		<Reserve1<Test>>::put(2);

		assert_noop!(SocialSwap2::swap(
			Origin::signed(ACCOUNT1),
			3,
			3,
			ACCOUNT3,
			"0x".encode()
		),
		Error::<Test>::InsufficientLiquidity);

		SocialSwap2::initialize(Origin::root(), FEE_TO, ADDRESS0, TREASURY, TOKEN0, TOKEN0);

		assert_noop!(SocialSwap2::swap(
			Origin::signed(ACCOUNT1),
			1,
			1,
			TOKEN0,
			"0x".encode()
		),
		Error::<Test>::InvalidTo);

	});
}

#[test]
fn test_swap_should_work() {
	new_test_ext().execute_with(|| {
		let token_0_amount: u128  = 5_000_000_000_000_000_000;
		let token_1_amount: u128  = 10_000_000_000_000_000_000;
		add_liquidity(token_0_amount, token_1_amount);
		let expected_output_amount: u128 = 1662497915624478906u128;
		let sawp_amount: u128  = 1_000_000_000_000_000_000;

		pallet_assets::Module::<Test>::transfer(&ASSET_ID, &ACCOUNT1, &TOKEN0, sawp_amount);

		assert_eq!(SocialSwap2::swap(
			Origin::signed(ACCOUNT1),
			0,
			expected_output_amount,
			ACCOUNT2,
			"0x".encode()
		),
				   Ok(())
		);

		assert_eq!(
			SocialSwap2::reserve0(),
			token_0_amount + sawp_amount
		);

		assert_eq!(
			SocialSwap2::reserve1(),
			token_1_amount - expected_output_amount
		);

		assert_eq!(
			pallet_assets::Module::<Test>::balance(ASSET_ID, TOKEN0),
			token_0_amount + sawp_amount
		);

		assert_eq!(
			pallet_assets::Module::<Test>::balance(ASSET_ID, TOKEN1),
			token_1_amount - expected_output_amount
		);

		assert_eq!(
			pallet_assets::Module::<Test>::balance(ASSET_ID, ACCOUNT2),
			pallet_assets::Module::<Test>::total_supply(ASSET_ID) + expected_output_amount
			- MINIMUM_LIQUIDITY as u128
		);
	});
}


fn add_liquidity(token_0_amount: u128, token_1_amount: u128) {
	SocialSwap2::initialize(Origin::root(), FEE_TO, ADDRESS0, TREASURY, TOKEN0, TOKEN1);
	pallet_assets::Module::<Test>::transfer(&ASSET_ID, &ACCOUNT1, &TOKEN0, token_0_amount);
	pallet_assets::Module::<Test>::transfer(&ASSET_ID, &ACCOUNT1, &TOKEN1, token_1_amount);
	assert_eq!(
		SocialSwap2::mint(
			Origin::signed(ACCOUNT1),
			ACCOUNT2
		),
		Ok(())
	);
}

