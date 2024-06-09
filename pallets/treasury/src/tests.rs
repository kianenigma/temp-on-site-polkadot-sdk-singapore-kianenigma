use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok, traits::fungibles};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(Treasury::do_something(RuntimeOrigin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(Something::<Test>::get(), Some(42));
		// Assert that the correct event was deposited
		System::assert_last_event(Event::SomethingStored { something: 42, who: 1 }.into());
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(Treasury::cause_error(RuntimeOrigin::signed(1)), Error::<Test>::NoneValue);
	});
}

#[test]
fn handle_assets() {
	new_test_ext().execute_with(|| {
		let alice = 1;
		let asset_id = 1337;

		// These are some easy configuration you can use when creating a new token...
		// Don't worry too much about the details here, just know this works.
		let admin = 0;
		let is_sufficient = true;
		let min_balance = 1;

		// Here we show that alice initially does not have any balance of some random asset... as
		// expected.
		assert_eq!(<Test as Config>::Fungibles::balance(asset_id, &alice), 0);

		// Before we can give alice any asset, we must first CREATE that asset in our system. Think
		// about this similar to someone launching a contract on Ethereum. Before they launch the
		// contract, there is no token. For tests, we assume people have created other tokens like
		// BTC, USDC, etc...
		assert_ok!(<<Test as Config>::Fungibles as fungibles::Create<_>>::create(
			asset_id,
			admin,
			is_sufficient,
			min_balance
		));

		// Now that the asset is created, we can mint some balance into the alice account
		assert_ok!(<<Test as Config>::Fungibles as fungibles::Mutate<_>>::mint_into(
			asset_id, &alice, 100
		));

		// And here we can see that alice has this balance.
		assert_eq!(<Test as Config>::Fungibles::balance(asset_id, &alice), 100);
	});
}
