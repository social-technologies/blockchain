#![cfg(test)]

use super::mock::{new_test_ext, Erc721, Origin, Test, USER_A, USER_B, USER_C, ROYALTY};
use super::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;

#[test]
fn mint_burn_tokens() {
    new_test_ext().execute_with(|| {
        let id_a: U256 = 1.into();
        let id_b: U256 = 2.into();
        let metadata_a: Vec<u8> = vec![1, 2, 3];
        let metadata_b: Vec<u8> = vec![4, 5, 6];

        assert_ok!(Erc721::mint(
            Origin::signed(1), // Origin::root(),
            USER_A,
            id_a,
            metadata_a.clone(),
			ROYALTY
        ));
        assert_eq!(
            Erc721::tokens(id_a).unwrap(),
            Erc721Token {
                id: id_a,
                metadata: metadata_a.clone(),
				royalty: ROYALTY
            }
        );
        assert_eq!(Erc721::token_count(), 1.into());
        assert_eq!(Erc721::max_token_id(), 1.into());
        assert_noop!(
            Erc721::mint(Origin::signed(1), USER_A, id_a, metadata_a, ROYALTY), // Erc721::mint(Origin::root(), USER_A, id_a, metadata_a),
            Error::<Test>::TokenAlreadyExists
        );

        assert_ok!(Erc721::mint(
            Origin::signed(1), // Origin::root(),
            USER_A,
            id_b,
            metadata_b.clone(),
			ROYALTY
        ));
        assert_eq!(
            Erc721::tokens(id_b).unwrap(),
            Erc721Token {
                id: id_b,
                metadata: metadata_b.clone(),
				royalty: ROYALTY
            }
        );
        assert_eq!(Erc721::token_count(), 2.into());
        assert_eq!(Erc721::max_token_id(), 2.into());
        assert_noop!(
            Erc721::mint(Origin::signed(1), USER_A, id_b, metadata_b, ROYALTY), // Erc721::mint(Origin::root(), USER_A, id_b, metadata_b),
            Error::<Test>::TokenAlreadyExists
        );

        assert_ok!(Erc721::burn(Origin::signed(1), id_a)); // assert_ok!(Erc721::burn(Origin::root(), id_a));
        assert_eq!(Erc721::token_count(), 1.into());
        assert_eq!(Erc721::max_token_id(), 2.into());
        assert!(!<Tokens<Test>>::contains_key(&id_a));
        assert!(!<TokenCreatorAndOwner<Test>>::contains_key(&id_a));

        assert_ok!(Erc721::burn(Origin::signed(1), id_b)); // assert_ok!(Erc721::burn(Origin::root(), id_b));
        assert_eq!(Erc721::token_count(), 0.into());
        assert_eq!(Erc721::max_token_id(), 2.into());
        assert!(!<Tokens<Test>>::contains_key(&id_b));
        assert!(!<TokenCreatorAndOwner<Test>>::contains_key(&id_b));
    })
}

#[test]
fn transfer_tokens() {
    new_test_ext().execute_with(|| {
        let id_a: U256 = 1.into();
        let id_b: U256 = 2.into();
        let metadata_a: Vec<u8> = vec![1, 2, 3];
        let metadata_b: Vec<u8> = vec![4, 5, 6];

        assert_ok!(Erc721::mint(Origin::signed(1), USER_A, id_a, metadata_a, ROYALTY)); // assert_ok!(Erc721::mint(Origin::root(), USER_A, id_a, metadata_a));
        assert_ok!(Erc721::mint(Origin::signed(1), USER_A, id_b, metadata_b, ROYALTY)); // assert_ok!(Erc721::mint(Origin::root(), USER_A, id_b, metadata_b));

        assert_ok!(Erc721::transfer(Origin::signed(USER_A), USER_B, id_a));
        assert_eq!(Erc721::owner_of(id_a).1, USER_B);

        assert_ok!(Erc721::transfer(Origin::signed(USER_A), USER_C, id_b));
        assert_eq!(Erc721::owner_of(id_b).1, USER_C);

        assert_ok!(Erc721::transfer(Origin::signed(USER_B), USER_A, id_a));
        assert_eq!(Erc721::owner_of(id_a).1, USER_A);

        assert_ok!(Erc721::transfer(Origin::signed(USER_C), USER_A, id_b));
        assert_eq!(Erc721::owner_of(id_b).1, USER_A);
    })
}
