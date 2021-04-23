use crate::{mock::*, Error};
use super::*;

#[test]
fn test_mint() {
	new_test_ext().execute_with(|| {

		SocialSwap2::initialize(Origin::root(), FEE_TO, ADDRESS0, TREASURY, TOKEN0, TOKEN1);
		let token_0_amount: u128  = 1_000_000_000_000_000_000;
		let token_1_amount: u128  = 4_000_000_000_000_000_000;
		pallet_assets::Module::<Test>::transfer(&ASSET_ID, &OWNER, &TOKEN0, token_0_amount);
		pallet_assets::Module::<Test>::transfer(&ASSET_ID, &OWNER, &TOKEN1, token_1_amount);
		let expected_liquidity: u128 = 2_000_000_000_000_000_000u128;

		assert_eq!(
			pallet_assets::Module::<Test>::total_supply(ASSET_ID),
			0
		);
		assert_eq!(
			SocialSwap2::mint(
				Origin::signed(OWNER),
				2
			),
			Ok(())
		);

		assert_eq!(
			pallet_assets::Module::<Test>::total_supply(ASSET_ID),
			expected_liquidity
		);

		assert_eq!(
			pallet_assets::Module::<Test>::balance(ASSET_ID, 2),
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


