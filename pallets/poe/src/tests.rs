use super::*;
use crate::{mock::*, Error};
use frame_support::{ assert_noop, assert_ok, BoundedVec};

// 创建存证 查看存储 重复创建 正常撤回，撤回失败
#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		// 存储项断言
		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);

		// 重复创建失败 已经存在
		let claim2 = vec![0, 1];
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim2.clone()),
			Error::<Test>::ProofAlreadyExist
		);

	})
}
// 撤销存证
#[test]
fn revoke_claim() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		// 存储项断言
		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
		// 撤回存证
		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));

		// 撤回失败不存在的claim
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimNotExist
		);
	});
}
// 正常转移存证
#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 3, 4];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2));
	});
}

// 转移存证失败； 不存在的存证，不是存证所有者
#[test]
fn transfer_claim_failed() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 3, 4];
		// 不存在错误
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2),
			Error::<Test>::ClaimNotExist);

		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(3), claim.clone(), 2),
			Error::<Test>::NostClaimOwner
		);
	});
}
