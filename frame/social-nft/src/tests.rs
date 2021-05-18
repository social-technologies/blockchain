#![cfg(test)]

use super::mock::{new_test_ext, SocialNft, Origin, Test, USER_A, USER_B, USER_C, ROYALTY};
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

        assert_eq!(SocialNft::balance_of(USER_C), 0.into());
        assert_ok!(SocialNft::mint(
            Origin::signed(USER_C),
            USER_A,
            id_a,
            metadata_a.clone(),
			ROYALTY
        ));
        assert_eq!(
            SocialNft::tokens(id_a).unwrap(),
            Erc721Token {
                id: id_a,
                metadata: metadata_a.clone(),
				royalty: ROYALTY
            }
        );
        assert_eq!(SocialNft::token_count(), 1.into());
        assert_eq!(SocialNft::max_token_id(), 1.into());
        assert_eq!(SocialNft::balance_of(USER_A), 1.into());
        assert_eq!(SocialNft::balance_of(USER_C), 0.into());
        assert_noop!(
            SocialNft::mint(Origin::signed(USER_C), USER_A, id_a, metadata_a, ROYALTY),
            Error::<Test>::TokenAlreadyExists
        );

        assert_ok!(SocialNft::mint(
            Origin::signed(USER_C),
            USER_A,
            id_b,
            metadata_b.clone(),
			ROYALTY
        ));
        // assert_eq!(
        //     SocialNft::tokens(id_b).unwrap(),
        //     Erc721Token {
        //         id: id_b,
        //         metadata: metadata_b.clone(),
		// 		royalty: ROYALTY
        //     }
        // );
        assert_eq!(SocialNft::token_count(), 2.into());
        assert_eq!(SocialNft::max_token_id(), 2.into());
        assert_noop!(
            SocialNft::mint(Origin::signed(1), USER_A, id_b, metadata_b, ROYALTY), // SocialNft::mint(Origin::root(), USER_A, id_b, metadata_b),
            Error::<Test>::TokenAlreadyExists
        );

        assert_ok!(SocialNft::burn(Origin::signed(1), id_a)); // assert_ok!(SocialNft::burn(Origin::root(), id_a));
        assert_eq!(SocialNft::token_count(), 1.into());
        assert_eq!(SocialNft::max_token_id(), 2.into());
        assert!(!<Tokens<Test>>::contains_key(&id_a));
        assert!(!<TokenCreatorAndOwner<Test>>::contains_key(&id_a));

        assert_ok!(SocialNft::burn(Origin::signed(1), id_b)); // assert_ok!(SocialNft::burn(Origin::root(), id_b));
        assert_eq!(SocialNft::token_count(), 0.into());
        assert_eq!(SocialNft::max_token_id(), 2.into());
        assert!(!<Tokens<Test>>::contains_key(&id_b));
        assert!(!<TokenCreatorAndOwner<Test>>::contains_key(&id_b));
        assert_eq!(SocialNft::balance_of(USER_A), 2.into());
        assert_eq!(SocialNft::balance_of(USER_C), 0.into());
        assert_noop!(
            SocialNft::mint(Origin::signed(USER_C), USER_A, id_b, metadata_b, ROYALTY),
            Error::<Test>::TokenAlreadyExists
        );

        assert_ok!(SocialNft::approve(Origin::signed(USER_A), USER_C, id_a));
        assert_eq!(SocialNft::token_approvals(id_a), USER_C);

        assert_ok!(SocialNft::burn(Origin::signed(USER_A), id_a));
        assert_eq!(SocialNft::token_count(), 1.into());
        assert_eq!(SocialNft::max_token_id(), 2.into());
        assert_eq!(SocialNft::balance_of(USER_A), 1.into());
        assert_eq!(SocialNft::balance_of(USER_C), 0.into());
        assert!(!<Tokens<Test>>::contains_key(&id_a));
        assert!(!<TokenCreatorAndOwner<Test>>::contains_key(&id_a));
        // Must be cleaning of approvals from the previous owner
        assert!(!<TokenApprovals<Test>>::contains_key(id_a));

        assert_ok!(SocialNft::burn(Origin::signed(USER_A), id_b));
        assert_eq!(SocialNft::token_count(), 0.into());
        assert_eq!(SocialNft::max_token_id(), 2.into());
        assert_eq!(SocialNft::balance_of(USER_A), 0.into());
        assert_eq!(SocialNft::balance_of(USER_C), 0.into());
        assert!(!<Tokens<Test>>::contains_key(&id_b));
        assert!(!<TokenCreatorAndOwner<Test>>::contains_key(&id_b));
    })
}

#[test]
//fn burn_from_not_owner_should_not_work() {
//    new_test_ext().execute_with(|| {
//        let id_a: U256 = 1.into();
//        let metadata_a: Vec<u8> = vec![1, 2, 3];
//
//        assert_eq!(Erc721::balance_of(USER_C), 0.into());
//        assert_ok!(Erc721::mint(
//            Origin::signed(USER_C),
//            USER_A,
//            id_a,
//            metadata_a.clone()
//        ));
//        assert_eq!(
//            Erc721::tokens(id_a).unwrap(),
//            Erc721Token {
//                id: id_a,
//                metadata: metadata_a.clone()
//            }
//        );
//        assert_eq!(Erc721::token_count(), 1.into());
//        assert_eq!(Erc721::max_token_id(), 1.into());
//        assert_eq!(Erc721::balance_of(USER_A), 1.into());
//        assert_eq!(Erc721::balance_of(USER_C), 0.into());
//
//        assert_noop!(
//            Erc721::burn(Origin::signed(USER_C), id_a),
//            Error::<Test>::NotOwner,
//        );
//   })
// }

#[test]
fn transfer_tokens_should_work() {
    new_test_ext().execute_with(|| {
        let id_a: U256 = 1.into();
        let id_b: U256 = 2.into();
        let metadata_a: Vec<u8> = vec![1, 2, 3];
        let metadata_b: Vec<u8> = vec![4, 5, 6];

        assert_ok!(SocialNft::mint(Origin::signed(1), USER_A, id_a, metadata_a, ROYALTY)); // assert_ok!(SocialNft::mint(Origin::root(), USER_A, id_a, metadata_a));
        assert_ok!(SocialNft::mint(Origin::signed(1), USER_A, id_b, metadata_b, ROYALTY)); // assert_ok!(SocialNft::mint(Origin::root(), USER_A, id_b, metadata_b));

        assert_ok!(SocialNft::transfer(Origin::signed(USER_A), USER_B, id_a));
        assert_eq!(SocialNft::owner_of(id_a).1, USER_B);

        assert_ok!(SocialNft::transfer(Origin::signed(USER_A), USER_C, id_b));
        assert_eq!(SocialNft::owner_of(id_b).1, USER_C);

        assert_ok!(SocialNft::transfer(Origin::signed(USER_B), USER_A, id_a));
        assert_eq!(SocialNft::owner_of(id_a).1, USER_A);

        assert_ok!(SocialNft::transfer(Origin::signed(USER_C), USER_A, id_b));
        assert_eq!(SocialNft::owner_of(id_b).1, USER_A);
    })
}

#[test]
fn set_ask_tokens() {
	new_test_ext().execute_with(|| {
		let id_a: U256 = 1.into();
		let metadata_a: Vec<u8> = vec![1, 2, 3];
		assert_noop!(
            SocialNft::set_ask(Origin::signed(1), id_a, 1, 1),
            Error::<Test>::NftIdDoesNotExist
        );

        assert_eq!(SocialNft::balance_of(USER_C), 0.into());
        assert_ok!(SocialNft::mint(
            Origin::signed(USER_C),
            USER_A,
            id_a,
            metadata_a.clone(),
			ROYALTY
        ));

		assert_noop!(
            SocialNft::set_ask(Origin::signed(11), id_a, 1, 1),
            Error::<Test>::NotOwner
        );

		assert_ok!(
            SocialNft::set_ask(Origin::signed(USER_A), id_a, 1, 1)
		);

        // assert_eq!(
        //     SocialNft::tokens(id_a).unwrap(),
        //     Erc721Token {
        //         id: id_a,
        //         metadata: metadata_a
        //     }
        // );
        assert_eq!(SocialNft::token_count(), 1.into());
        assert_eq!(SocialNft::max_token_id(), 1.into());
        assert_eq!(SocialNft::balance_of(USER_A), 1.into());
        assert_eq!(SocialNft::balance_of(USER_C), 0.into());

        assert_noop!(
            SocialNft::burn(Origin::signed(USER_C), id_a),
            Error::<Test>::NotOwner,
        );

		assert!(<TokenAskAmount<Test>>::contains_key(id_a, 1));
	})
}

#[test]
fn set_bid_tokens() {
	new_test_ext().execute_with(|| {
		let id_a: U256 = 1.into();
		let metadata_a: Vec<u8> = vec![1, 2, 3];
		assert_noop!(
            SocialNft::set_bid(Origin::signed(1), id_a, 1, 1, 1),
            Error::<Test>::NftIdDoesNotExist
        );

		assert_ok!(SocialNft::mint(
            Origin::signed(1), // Origin::root(),
            USER_A,
            id_a,
            metadata_a.clone(),
			ROYALTY
        ));

		assert_ok!(
            SocialNft::set_ask(Origin::signed(USER_A), id_a, 1, 0)
        );

		assert_noop!(
            SocialNft::set_bid(Origin::signed(USER_A), id_a, 1, 1, 1),
            Error::<Test>::NotForSale
        );

		assert_ok!(
            SocialNft::set_ask(Origin::signed(USER_A), id_a, 1, 1)
        );

		assert_ok!(
            SocialNft::set_bid(Origin::signed(USER_A), id_a, 1, 1, 1)
        );

		assert!(<TokenBidAmount<Test>>::contains_key(id_a, 1));
	})
}

#[test]
fn remove_bid_tokens() {
	new_test_ext().execute_with(|| {
		let id_a: U256 = 1.into();
		let metadata_a: Vec<u8> = vec![1, 2, 3];

		assert_noop!(
            SocialNft::remove_bid(Origin::signed(1), id_a),
            Error::<Test>::NftIdDoesNotExist
        );

		assert_ok!(SocialNft::mint(
            Origin::signed(1), // Origin::root(),
            USER_A,
            id_a,
            metadata_a.clone(),
			ROYALTY
        ));

		assert_ok!(
            SocialNft::set_ask(Origin::signed(USER_A), id_a, 1, 1)
        );

		assert_ok!(
            SocialNft::set_bid(Origin::signed(USER_A), id_a, 1, 1, 1)
        );

		assert_ok!(
            SocialNft::remove_bid(Origin::signed(1), id_a),
        );

		assert!(!SocialNft::bid_token(id_a, 1).unwrap().is_active);
	})
}

fn transfer_tokens_from_not_owner_should_not_work() {
    new_test_ext().execute_with(|| {
        let id_a: U256 = 1.into();
        let metadata_a: Vec<u8> = vec![1, 2, 3];

        assert_ok!(SocialNft::mint(
            Origin::signed(USER_C),
            USER_A,
            id_a,
            metadata_a,
			ROYALTY
        ));
        assert_eq!(SocialNft::owner_of(id_a).0, USER_A);

        assert_noop!(
            SocialNft::transfer(Origin::signed(USER_C), USER_B, id_a),
            Error::<Test>::NotOwner
        );
        assert_eq!(SocialNft::owner_of(id_a).0, USER_A);
    })
}

#[test]
fn set_approval_for_all_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(SocialNft::is_approved_for_all(USER_C, USER_A), false);
        assert_ok!(SocialNft::set_approval_for_all(
            Origin::signed(USER_C),
            USER_A,
            true
        ));
        assert_eq!(SocialNft::is_approved_for_all(USER_C, USER_A), true);
        assert_ok!(SocialNft::set_approval_for_all(
            Origin::signed(USER_C),
            USER_A,
            false
        ));
        assert_eq!(SocialNft::is_approved_for_all(USER_C, USER_A), false);
    })
}

#[test]
fn set_approval_for_all_for_same_account_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(SocialNft::is_approved_for_all(USER_C, USER_A), false);
        assert_noop!(
            SocialNft::set_approval_for_all(Origin::signed(USER_C), USER_C, true),
            Error::<Test>::ApproveToCaller
        );
        assert_eq!(SocialNft::is_approved_for_all(USER_C, USER_A), false);
    })
}

#[test]
fn approve_should_work() {
    new_test_ext().execute_with(|| {
        let id_a: U256 = 1.into();
        let id_b: U256 = 2.into();
        let metadata_a: Vec<u8> = vec![1, 2, 3];
        let metadata_b: Vec<u8> = vec![4, 5, 6];

        assert_ok!(SocialNft::mint(
            Origin::signed(USER_C),
            USER_A,
            id_a,
            metadata_a,
			ROYALTY
        ));
        assert_eq!(SocialNft::owner_of(id_a).0, USER_A);
        assert_ok!(SocialNft::approve(Origin::signed(USER_A), USER_C, id_a));
        assert_eq!(SocialNft::token_approvals(id_a), USER_C);

        assert_ok!(SocialNft::mint(
            Origin::signed(USER_C),
            USER_B,
            id_b,
            metadata_b,
			ROYALTY
        ));
        assert_eq!(SocialNft::owner_of(id_a).0, USER_A);
        assert_ok!(SocialNft::set_approval_for_all(
            Origin::signed(USER_B),
            USER_A,
            true
        ));
        assert_ok!(SocialNft::approve(Origin::signed(USER_A), USER_C, id_b));
        assert_eq!(SocialNft::token_approvals(id_a), USER_C);
    })
}

#[test]
fn approve_for_same_account_should_not_work() {
    new_test_ext().execute_with(|| {
        let id_a: U256 = 1.into();
        let metadata_a: Vec<u8> = vec![1, 2, 3];

        assert_ok!(SocialNft::mint(
            Origin::signed(USER_C),
            USER_C,
            id_a,
            metadata_a
        ));
        assert_eq!(SocialNft::owner_of(id_a).unwrap(), USER_C);
        assert_noop!(
            SocialNft::approve(Origin::signed(USER_C), USER_C, id_a),
            Error::<Test>::ApprovalToCurrentOwner
        );
        assert!(!<TokenApprovals<Test>>::contains_key(id_a));
    })
}
