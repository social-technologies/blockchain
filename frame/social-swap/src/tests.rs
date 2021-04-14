use crate::{mock::*, Error};
use frame_support::assert_noop;
use super::*;

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
	});
}
