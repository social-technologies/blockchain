use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn transfering_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(MissionTokens::balance(1, 1), 0);
        MissionTokens::mint(1, 1, 100);
        assert_eq!(MissionTokens::balance(1, 1), 100);
        assert_eq!(MissionTokens::balance(1, 2), 0);
        assert_ok!(MissionTokens::transfer(Origin::signed(1), 1, 2, 50));
        assert_eq!(MissionTokens::balance(1, 1), 50);
        assert_eq!(MissionTokens::balance(1, 2), 50);
    });
}

#[test]
fn minting_and_burning_should() {
    new_test_ext().execute_with(|| {
        assert_eq!(MissionTokens::balance(1, 1), 0);
        MissionTokens::mint(1, 1, 100);
        assert_eq!(MissionTokens::balance(1, 1), 100);
        MissionTokens::burn(1, 1, 50);
        assert_eq!(MissionTokens::balance(1, 1), 50);
    });
}

#[test]
fn transfering_zero_amount_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            MissionTokens::transfer(Origin::signed(1), 1, 2, 0),
            Error::<Test>::AmountZero
        );
    });
}

#[test]
fn transferring_amount_more_than_available_balance_should_not_work() {
    new_test_ext().execute_with(|| {
        MissionTokens::mint(1, 1, 100);
        assert_noop!(
            MissionTokens::transfer(Origin::signed(1), 1, 2, 150),
            Error::<Test>::BalanceLow
        );
    });
}
