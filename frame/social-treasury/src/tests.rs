use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        assert_ok!(SocialTreasury::do_something(Origin::signed(1), 42));
        assert_eq!(SocialTreasury::something(), Some(42));
    });
}

#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            SocialTreasury::cause_error(Origin::signed(1)),
            Error::<Test>::NoneValue
        );
    });
}
