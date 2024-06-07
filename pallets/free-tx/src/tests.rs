use crate::{mock::*, *};
use frame_support::{
	assert_noop, assert_ok,
	traits::{
		fungible::{Inspect, InspectHold, Mutate},
		Get,
	},
};
use sp_runtime::TokenError;

#[test]
fn it_works() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(FreeTx::free_tx(RuntimeOrigin::signed(1), true));
		// Assert that the correct event was deposited
		System::assert_last_event(Event::TxSuccess.into());
		// Check error case
		assert_noop!(FreeTx::free_tx(RuntimeOrigin::signed(1), false), Error::<Test>::TxFailed);
	});
}

#[test]
fn end_to_end_hold_works() {
	new_test_ext().execute_with(|| {
		let alice = 1;

		// Alice tries to hold some funds, but has none... so it fails.
		assert_noop!(
			FreeTx::hold_my_funds(RuntimeOrigin::signed(alice)),
			TokenError::FundsUnavailable
		);

		// Alice has no balance according to the balances pallet:
		assert_eq!(<Test as Config>::NativeBalance::balance(&alice), 0);

		// Alice needs some money for our test to make sense;
		<Test as Config>::NativeBalance::set_balance(&alice, 100_000);

		// Now alice has some money
		assert_eq!(<Test as Config>::NativeBalance::balance(&alice), 100_000);

		// But no balance held.
		assert_eq!(
			<Test as Config>::NativeBalance::balance_on_hold(
				&HoldReason::FreeTxHold.into(),
				&alice
			),
			0
		);

		// Now Alice can hold funds.
		assert_ok!(FreeTx::hold_my_funds(RuntimeOrigin::signed(alice)));

		// Local Storage is updated
		assert_eq!(AmountHeld::<Test>::get(&alice), <Test as Config>::HoldAmount::get());

		// The amount held is also being represented in the balances pallet.
		assert_eq!(
			<Test as Config>::NativeBalance::balance_on_hold(
				&HoldReason::FreeTxHold.into(),
				&alice
			),
			1337
		);

		// Alice cannot call `hold_my_funds` again.
		assert_noop!(
			FreeTx::hold_my_funds(RuntimeOrigin::signed(alice)),
			Error::<Test>::AlreadyHeld
		);

		// Alice can release her funds.
		assert_ok!(FreeTx::release_my_funds(RuntimeOrigin::signed(alice)));

		// Now alice has no held balance again
		assert_eq!(
			<Test as Config>::NativeBalance::balance_on_hold(
				&HoldReason::FreeTxHold.into(),
				&alice
			),
			0
		);

		// Alice cannot release her funds again.
		assert_noop!(
			FreeTx::release_my_funds(RuntimeOrigin::signed(alice)),
			Error::<Test>::NothingHeld
		);
	});
}
