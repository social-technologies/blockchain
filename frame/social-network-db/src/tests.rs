use crate::{mock::*, Error, Judgement, Registration};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};

#[test]
fn adding_registrar_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(UsernameRegistry::registrars(), vec![]);
        assert_noop!(
            UsernameRegistry::add_registrar(Origin::signed(2), 3),
            DispatchError::BadOrigin,
        );
        assert_ok!(UsernameRegistry::add_registrar(Origin::signed(1), 3));
        assert_eq!(UsernameRegistry::registrars(), vec![Some(3)]);
    });
}

#[test]
fn registration_and_unregistration_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(UsernameRegistry::add_registrar(Origin::signed(1), 3));
        assert_eq!(UsernameRegistry::registration_of(b"foo".to_vec()), None);
        assert_noop!(
            UsernameRegistry::register(Origin::signed(4), b"foo".to_vec(), 1),
            Error::<Test>::EmptyIndex,
        );
        assert_ok!(UsernameRegistry::register(
            Origin::signed(4),
            b"foo".to_vec(),
            0
        ));
        assert_noop!(
            UsernameRegistry::register(Origin::signed(4), b"foo".to_vec(), 0),
            Error::<Test>::UsernameAlreadyRegistered,
        );
        assert_noop!(
            UsernameRegistry::register(Origin::signed(4), b"bar".to_vec(), 0),
            Error::<Test>::AccountAlreadyRegistered,
        );
        assert_eq!(
            UsernameRegistry::registration_of(b"foo".to_vec()),
            Some(Registration {
                judgements: vec![(0, Judgement::Requested)],
                account_id: 4
            }),
        );
        assert_eq!(UsernameRegistry::account(4), Some(b"foo".to_vec()));
        assert_noop!(
            UsernameRegistry::unregister(Origin::signed(5), b"foo".to_vec()),
            Error::<Test>::UnregisterForbidden,
        );
        assert_noop!(
            UsernameRegistry::unregister(Origin::signed(4), b"bar".to_vec()),
            Error::<Test>::UsernameNotFound,
        );
        assert_ok!(UsernameRegistry::unregister(
            Origin::signed(4),
            b"foo".to_vec()
        ));
        assert_eq!(UsernameRegistry::registration_of(b"foo".to_vec()), None);
        assert_eq!(UsernameRegistry::account(4), None);
    });
}

#[test]
fn killing_username_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(UsernameRegistry::add_registrar(Origin::signed(1), 3));
        assert_eq!(UsernameRegistry::registration_of(b"foo".to_vec()), None);
        assert_ok!(UsernameRegistry::register(
            Origin::signed(4),
            b"foo".to_vec(),
            0
        ));
        assert_eq!(
            UsernameRegistry::registration_of(b"foo".to_vec()),
            Some(Registration {
                judgements: vec![(0, Judgement::Requested)],
                account_id: 4
            }),
        );
        assert_noop!(
            UsernameRegistry::kill_username(Origin::signed(3), b"bar".to_vec()),
            DispatchError::BadOrigin,
        );
        assert_noop!(
            UsernameRegistry::kill_username(Origin::signed(2), b"bar".to_vec()),
            Error::<Test>::UsernameNotFound,
        );
        assert_ok!(UsernameRegistry::kill_username(
            Origin::signed(2),
            b"foo".to_vec()
        ));
        assert_eq!(UsernameRegistry::registration_of(b"foo".to_vec()), None);
        assert_eq!(UsernameRegistry::account(4), None);
    });
}

#[test]
fn providing_judgement_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(UsernameRegistry::add_registrar(Origin::signed(1), 3));
        assert_ok!(UsernameRegistry::add_registrar(Origin::signed(1), 4));
        assert_eq!(UsernameRegistry::registration_of(b"foo".to_vec()), None);
        assert_ok!(UsernameRegistry::register(
            Origin::signed(4),
            b"foo".to_vec(),
            0
        ));
        assert_eq!(
            UsernameRegistry::registration_of(b"foo".to_vec()),
            Some(Registration {
                judgements: vec![(0, Judgement::Requested)],
                account_id: 4
            }),
        );
        assert_noop!(
            UsernameRegistry::provide_judgement(
                Origin::signed(4),
                0,
                b"foo".to_vec(),
                Judgement::Approved
            ),
            Error::<Test>::InvalidIndex,
        );
        assert_noop!(
            UsernameRegistry::provide_judgement(
                Origin::signed(5),
                0,
                b"foo".to_vec(),
                Judgement::Approved
            ),
            Error::<Test>::InvalidIndex,
        );
        assert_ok!(UsernameRegistry::provide_judgement(
            Origin::signed(3),
            0,
            b"foo".to_vec(),
            Judgement::Approved
        ));
        assert_ok!(UsernameRegistry::provide_judgement(
            Origin::signed(4),
            1,
            b"foo".to_vec(),
            Judgement::Approved
        ));
        assert_eq!(
            UsernameRegistry::registration_of(b"foo".to_vec()),
            Some(Registration {
                judgements: vec![(0, Judgement::Approved), (1, Judgement::Approved)],
                account_id: 4
            }),
        );
    });
}

#[test]
fn registration_and_unregistration_with_invalid_username_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(UsernameRegistry::add_registrar(Origin::signed(1), 3));
        assert_eq!(UsernameRegistry::registration_of(b"123".to_vec()), None);
        assert_noop!(
            UsernameRegistry::register(Origin::signed(4), b"12".to_vec(), 0),
            Error::<Test>::UsernameIsVeryShort,
        );
        assert_noop!(
            UsernameRegistry::register(Origin::signed(4), b"12345678901".to_vec(), 0),
            Error::<Test>::UsernameIsVeryLong,
        );
        assert_noop!(
            UsernameRegistry::register(Origin::signed(4), b"123!@#".to_vec(), 0),
            Error::<Test>::UsernameHasInvalidChars,
        );
        assert_ok!(UsernameRegistry::register(
            Origin::signed(4),
            b"123".to_vec(),
            0
        ));
        assert_eq!(
            UsernameRegistry::registration_of(b"123".to_vec()),
            Some(Registration {
                judgements: vec![(0, Judgement::Requested)],
                account_id: 4
            }),
        );

        assert_noop!(
            UsernameRegistry::unregister(Origin::signed(4), b"12".to_vec()),
            Error::<Test>::UsernameIsVeryShort,
        );
        assert_noop!(
            UsernameRegistry::unregister(Origin::signed(4), b"12345678901".to_vec()),
            Error::<Test>::UsernameIsVeryLong,
        );
        assert_ok!(UsernameRegistry::unregister(
            Origin::signed(4),
            b"123".to_vec()
        ));
        assert_eq!(UsernameRegistry::registration_of(b"123".to_vec()), None);
    });
}
