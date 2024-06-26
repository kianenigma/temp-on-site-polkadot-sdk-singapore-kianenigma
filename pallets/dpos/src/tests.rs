use crate::{mock::*, *};
use frame_support::assert_ok;

#[test]
fn check_based_on_max_validators() {
	new_test_ext().execute_with(|| {
		assert_eq!(<Test as crate::Config>::MaxValidators::get(), 10);

		// change this `Get` impl on the fly -- this only works if MaxValidators is `pub static`.
		crate::mock::MaxValidators::set(42);

		assert_eq!(<Test as crate::Config>::MaxValidators::get(), 42);
	})
}

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		run_to_block(1);
		// Dispatch a signed extrinsic.
		assert_ok!(Dpos::do_something(RuntimeOrigin::signed(1), 43));
		// Read pallet storage and assert an expected result.
		assert_eq!(Something::<Test>::get(), Some(43));
		// Assert that the correct event was deposited
		System::assert_last_event(Event::SomethingStored { something: 43, who: 1 }.into());
	});
}

#[test]
fn test_custom_author_in_test() {
	use frame_support::traits::FindAuthor;
	// initially, our author is 7, as defined in the mock file.
	// Now our pallet will read 8 as the block author
	assert_eq!(
		<Test as crate::Config>::FindAuthor::find_author::<Vec<_>>(Default::default()),
		Some(7)
	);

	mock::Author::set(8);

	// Now our pallet will read 8 as the block author
	assert_eq!(
		<Test as crate::Config>::FindAuthor::find_author::<Vec<_>>(Default::default()),
		Some(8)
	);
}
