#![cfg(test)]

use super::mock::{new_test_ext, Erc721, Origin, Test, USER_A, USER_B, USER_C};
use super::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;

#[test]
fn mint_and_burn_tokens_should_work() {
    new_test_ext().execute_with(|| {
        let id_a: U256 = 1.into();
        let id_b: U256 = 2.into();
        let metadata_a: Vec<u8> = vec![1, 2, 3];
        let metadata_b: Vec<u8> = vec![4, 5, 6];

        assert_eq!(Erc721::balance_of(USER_C), 0.into());
        assert_ok!(Erc721::mint(
            Origin::signed(USER_C),
            USER_A,
            id_a,
            metadata_a.clone()
        ));
        assert_eq!(
            Erc721::tokens(id_a).unwrap(),
            Erc721Token {
                id: id_a,
                metadata: metadata_a.clone()
            }
        );
        assert_eq!(Erc721::token_count(), 1.into());
        assert_eq!(Erc721::max_token_id(), 1.into());
        assert_eq!(Erc721::balance_of(USER_A), 1.into());
        assert_eq!(Erc721::balance_of(USER_C), 0.into());
        assert_noop!(
            Erc721::mint(Origin::signed(USER_C), USER_A, id_a, metadata_a),
            Error::<Test>::TokenAlreadyExists
        );

        assert_ok!(Erc721::mint(
            Origin::signed(USER_C),
            USER_A,
            id_b,
            metadata_b.clone()
        ));
        assert_eq!(
            Erc721::tokens(id_b).unwrap(),
            Erc721Token {
                id: id_b,
                metadata: metadata_b.clone()
            }
        );
        assert_eq!(Erc721::token_count(), 2.into());
        assert_eq!(Erc721::max_token_id(), 2.into());
        assert_eq!(Erc721::balance_of(USER_A), 2.into());
        assert_eq!(Erc721::balance_of(USER_C), 0.into());
        assert_noop!(
            Erc721::mint(Origin::signed(USER_C), USER_A, id_b, metadata_b),
            Error::<Test>::TokenAlreadyExists
        );

        assert_ok!(Erc721::approve(Origin::signed(USER_A), USER_C, id_a));
        assert_eq!(Erc721::token_approvals(id_a), USER_C);

        assert_ok!(Erc721::burn(Origin::signed(USER_A), id_a));
        assert_eq!(Erc721::token_count(), 1.into());
        assert_eq!(Erc721::max_token_id(), 2.into());
        assert_eq!(Erc721::balance_of(USER_A), 1.into());
        assert_eq!(Erc721::balance_of(USER_C), 0.into());
        assert!(!<Tokens>::contains_key(&id_a));
        assert!(!<OwnerOf<Test>>::contains_key(&id_a));
        // Must be cleaning of approvals from the previous owner
        assert!(!<TokenApprovals<Test>>::contains_key(id_a));

        assert_ok!(Erc721::burn(Origin::signed(USER_A), id_b));
        assert_eq!(Erc721::token_count(), 0.into());
        assert_eq!(Erc721::max_token_id(), 2.into());
        assert_eq!(Erc721::balance_of(USER_A), 0.into());
        assert_eq!(Erc721::balance_of(USER_C), 0.into());
        assert!(!<Tokens>::contains_key(&id_b));
        assert!(!<OwnerOf<Test>>::contains_key(&id_b));
    })
}

#[test]
fn burn_from_not_owner_should_not_work() {
    new_test_ext().execute_with(|| {
        let id_a: U256 = 1.into();
        let metadata_a: Vec<u8> = vec![1, 2, 3];

        assert_eq!(Erc721::balance_of(USER_C), 0.into());
        assert_ok!(Erc721::mint(
            Origin::signed(USER_C),
            USER_A,
            id_a,
            metadata_a.clone()
        ));
        assert_eq!(
            Erc721::tokens(id_a).unwrap(),
            Erc721Token {
                id: id_a,
                metadata: metadata_a.clone()
            }
        );
        assert_eq!(Erc721::token_count(), 1.into());
        assert_eq!(Erc721::max_token_id(), 1.into());
        assert_eq!(Erc721::balance_of(USER_A), 1.into());
        assert_eq!(Erc721::balance_of(USER_C), 0.into());

        assert_noop!(
            Erc721::burn(Origin::signed(USER_C), id_a),
            Error::<Test>::NotOwner,
        );
    })
}

#[test]
fn transfer_tokens_should_work() {
    new_test_ext().execute_with(|| {
        let id_a: U256 = 1.into();
        let id_b: U256 = 2.into();
        let metadata_a: Vec<u8> = vec![1, 2, 3];
        let metadata_b: Vec<u8> = vec![4, 5, 6];

        assert_ok!(Erc721::mint(Origin::signed(USER_C), USER_A, id_a, metadata_a));
        assert_ok!(Erc721::mint(Origin::signed(USER_C), USER_A, id_b, metadata_b));

        assert_ok!(Erc721::approve(Origin::signed(USER_A), USER_C, id_a));
        assert_eq!(Erc721::token_approvals(id_a), USER_C);

        assert_ok!(Erc721::transfer(Origin::signed(USER_A), USER_B, id_a));
        assert_eq!(Erc721::owner_of(id_a).unwrap(), USER_B);
        // Must be cleaning of approvals from the previous owner
        assert!(!<TokenApprovals<Test>>::contains_key(id_a));

        assert_ok!(Erc721::transfer(Origin::signed(USER_A), USER_C, id_b));
        assert_eq!(Erc721::owner_of(id_b).unwrap(), USER_C);

        assert_ok!(Erc721::transfer(Origin::signed(USER_B), USER_A, id_a));
        assert_eq!(Erc721::owner_of(id_a).unwrap(), USER_A);

        assert_ok!(Erc721::transfer(Origin::signed(USER_C), USER_A, id_b));
        assert_eq!(Erc721::owner_of(id_b).unwrap(), USER_A);
    })
}

#[test]
fn transfer_tokens_from_not_owner_should_not_work() {
    new_test_ext().execute_with(|| {
        let id_a: U256 = 1.into();
        let metadata_a: Vec<u8> = vec![1, 2, 3];

        assert_ok!(Erc721::mint(Origin::signed(USER_C), USER_A, id_a, metadata_a));
        assert_eq!(Erc721::owner_of(id_a).unwrap(), USER_A);

        assert_noop!(
            Erc721::transfer(Origin::signed(USER_C), USER_B, id_a),
            Error::<Test>::NotOwner

        );
        assert_eq!(Erc721::owner_of(id_a).unwrap(), USER_A);
    })
}

#[test]
fn set_approval_for_all_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(Erc721::is_approved_for_all(USER_C, USER_A), false);
        assert_ok!(Erc721::set_approval_for_all(Origin::signed(USER_C), USER_A, true));
        assert_eq!(Erc721::is_approved_for_all(USER_C, USER_A), true);
        assert_ok!(Erc721::set_approval_for_all(Origin::signed(USER_C), USER_A, false));
        assert_eq!(Erc721::is_approved_for_all(USER_C, USER_A), false);
    })
}

#[test]
fn approve_should_work() {
    new_test_ext().execute_with(|| {
        let id_a: U256 = 1.into();
        let id_b: U256 = 2.into();
        let metadata_a: Vec<u8> = vec![1, 2, 3];
        let metadata_b: Vec<u8> = vec![4, 5, 6];

        assert_ok!(Erc721::mint(Origin::signed(USER_C), USER_A, id_a, metadata_a));
        assert_eq!(Erc721::owner_of(id_a).unwrap(), USER_A);
        assert_ok!(Erc721::approve(Origin::signed(USER_A), USER_C, id_a));
        assert_eq!(Erc721::token_approvals(id_a), USER_C);

        assert_ok!(Erc721::mint(Origin::signed(USER_C), USER_B, id_b, metadata_b));
        assert_eq!(Erc721::owner_of(id_a).unwrap(), USER_A);
        assert_ok!(Erc721::set_approval_for_all(Origin::signed(USER_B), USER_A, true));
        assert_ok!(Erc721::approve(Origin::signed(USER_A), USER_C, id_b));
        assert_eq!(Erc721::token_approvals(id_a), USER_C);
    })
}
