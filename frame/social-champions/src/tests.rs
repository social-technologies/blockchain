use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn registration_and_unregistration_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(ValidatorRegistry::champion_of(1), 0);
        assert_eq!(
            ValidatorRegistry::champions_of_social_token(10),
            Vec::<u64>::new()
        );
        assert_eq!(ValidatorRegistry::champions(), Vec::<u64>::new());
        assert_ok!(ValidatorRegistry::register(Origin::signed(1), 10));
        assert_eq!(ValidatorRegistry::champion_of(1), 10);
        assert_eq!(ValidatorRegistry::champions_of_social_token(10), vec![1]);
        assert_eq!(ValidatorRegistry::champions(), vec![1]);
        assert_ok!(ValidatorRegistry::register(Origin::signed(2), 10));
        assert_eq!(ValidatorRegistry::champion_of(2), 10);
        assert_eq!(ValidatorRegistry::champions_of_social_token(10), vec![1, 2]);
        assert_eq!(ValidatorRegistry::champions(), vec![1, 2]);
        assert_ok!(ValidatorRegistry::unregister(Origin::signed(2)));
        assert_eq!(ValidatorRegistry::champions_of_social_token(10), vec![1]);
        assert_eq!(ValidatorRegistry::champions(), vec![1]);
        assert_ok!(ValidatorRegistry::unregister(Origin::signed(1)));
        assert_eq!(ValidatorRegistry::champion_of(1), 0);
        assert_eq!(
            ValidatorRegistry::champions_of_social_token(10),
            Vec::<u64>::new()
        );
        assert_eq!(ValidatorRegistry::champions(), Vec::<u64>::new());
    });
}

#[test]
fn re_registration_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(ValidatorRegistry::register(Origin::signed(1), 10));
        assert_eq!(ValidatorRegistry::champion_of(1), 10);
        assert_eq!(ValidatorRegistry::champions_of_social_token(10), vec![1]);
        assert_eq!(ValidatorRegistry::champions(), vec![1]);
        assert_noop!(
            ValidatorRegistry::register(Origin::signed(1), 11),
            Error::<Test>::AlreadyRegistered
        );
        assert_eq!(ValidatorRegistry::champions_of_social_token(10), vec![1]);
        assert_eq!(ValidatorRegistry::champions(), vec![1]);
    });
}

#[test]
fn registration_with_invalid_social_id_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ValidatorRegistry::register(Origin::signed(1), 0),
            pallet_assets::Error::<Test>::Unknown
        );
        assert_noop!(
            ValidatorRegistry::register(Origin::signed(1), 100500),
            pallet_assets::Error::<Test>::Unknown
        );
    });
}

#[test]
fn unregistration_for_unregistered_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ValidatorRegistry::unregister(Origin::signed(1)),
            Error::<Test>::NotFound
        );
    });
}
