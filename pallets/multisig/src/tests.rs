use crate::{mock::*, *};
use frame_support::{
	assert_noop, assert_ok,
	pallet_prelude::Weight,
	traits::fungible::{Inspect, Mutate},
};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(Multisig::do_something(RuntimeOrigin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(Something::<Test>::get(), Some(42));
		// Assert that the correct event was deposited
		System::assert_last_event(Event::SomethingStored { something: 42, who: 1 }.into());
	});
}

#[test]
fn try_mutate_works_as_expected() {
	StateBuilder::default().build_and_execute(|| {
		assert_eq!(SomethingMap::<Test>::get(&10), None);

		assert_ok!(SomethingMap::<Test>::try_mutate::<_, _, (), _>(&10, |val| {
			*val = Some(42);
			Ok(())
		}));

		assert_eq!(SomethingMap::<Test>::get(&10), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(Multisig::cause_error(RuntimeOrigin::signed(1)), Error::<Test>::NoneValue);
	});
}

#[test]
fn redispatch_works() {
	new_test_ext().execute_with(|| {
		// You need to set the `block_number` to at least `1` for events to be emitted!
		System::set_block_number(1);

		let alice = 1;
		let bob = 2;
		let amount: u128 = 12345;

		// Take note what is happening here:
		// - remember the construction of the `pallet::Call` does not have `origin`, but the other
		//   params
		// - remember the `call` is an enum variant with the fields of the extrinsic
		// - remember you can convert a `pallet::Call` into the `RuntimeCall` because it implements
		//   `Into`
		let transfer_call: RuntimeCall =
			pallet_balances::Call::<Test>::transfer_keep_alive { dest: bob, value: amount }.into();

		// Our `redispatch` function will actually EXECUTE the balance transfer call here on behalf
		// of our `origin`, so to call this from alice, we need to give alice some balance.
		<Test as Config>::NativeBalance::set_balance(&alice, 100_000);

		assert_ok!(Multisig::redispatch(RuntimeOrigin::signed(alice), Box::new(transfer_call)));

		// This is the hard-coded weight for balances pallet with the `type WeightInfo = ()` impl.
		// I got this by letting rust error tell me what it is.
		let transfer_keep_alive_weight: Weight = Weight::from_parts(163159000, 3593);

		// We can check the events are correct, and has the weight we might expect.
		System::assert_last_event(Event::CallWeight { weight: transfer_keep_alive_weight }.into());

		// ALSO we can see the transfer happened!
		assert_eq!(<Test as Config>::NativeBalance::balance(&alice), 100_000 - 12345);
		assert_eq!(<Test as Config>::NativeBalance::balance(&bob), 12345);
	});
}
