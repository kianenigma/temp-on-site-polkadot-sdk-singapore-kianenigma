#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/polkadot_sdk/frame_runtime/index.html
// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html
// https://paritytech.github.io/polkadot-sdk/master/frame_support/attr.pallet.html#dev-mode-palletdev_mode
#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Zero,
		traits::{fungible, FindAuthor},
	};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	pub trait ReportNewValidatorSet<AccountId> {
		fn report_new_validator_set(_new_set: Vec<AccountId>) {}
	}

	pub type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type to access the Balances Pallet.
		type NativeBalance: fungible::Inspect<Self::AccountId>
			+ fungible::Mutate<Self::AccountId>
			+ fungible::hold::Inspect<Self::AccountId>
			+ fungible::hold::Mutate<Self::AccountId>
			+ fungible::freeze::Inspect<Self::AccountId>
			+ fungible::freeze::Mutate<Self::AccountId>;

		/// The maximum number of authorities that the pallet can hold.
		type MaxValidators: Get<u32>;

		/// Find the author of a block. A fake provide for this type is provided in the runtime. You
		/// can use a similar mechanism in your tests.
		type FindAuthor: FindAuthor<Self::AccountId>;

		/// Report the new validators to the runtime. This is done through a custom trait defined in
		/// this pallet.
		type ReportNewValidatorSet: ReportNewValidatorSet<Self::AccountId>;

		/// We use a configurable constant `BlockNumber` to tell us when we should trigger the
		/// validator set change. The runtime developer should implement this to represent the time
		/// they want validators to change, but for your pallet, you just care about the block
		/// number.
		#[pallet::constant]
		type EpochDuration: Get<BlockNumberFor<Self>>;
	}

	/// The pallet's storage items.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#storage
	/// https://paritytech.github.io/polkadot-sdk/master/frame_support/pallet_macros/attr.storage.html
	#[pallet::storage]
	pub type Something<T> = StorageValue<Value = u32>;
	#[pallet::storage]
	pub type SomethingMap<T: Config> = StorageMap<Key = T::AccountId, Value = BlockNumberFor<T>>;

	/// Take note of the different attributes needed on a custom structure so that it can be used in
	/// storage. You can add other derives if you need...
	#[derive(TypeInfo, Encode, Decode, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct DelegationInfo<T: Config> {
		// The validator who is getting the delegation
		pub who: T::AccountId,
		// The amount being delegated
		pub amount: BalanceOf<T>,
	}

	#[pallet::storage]
	pub type Delegations<T: Config> = StorageMap<Key = T::AccountId, Value = DelegationInfo<T>>;

	#[derive(TypeInfo, Encode, Decode, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct ValidatorBacking<T: Config> {
		who: T::AccountId,
		amount: BalanceOf<T>,
	}

	// This vector should always be sorted, with the lowest amount delegated at the end.
	// When a delegator delegates to a validator, and we increase their backing, we check if that
	// validator has more delegation than the last element in this vec.
	//
	// If so, we pop and swap.
	//
	// We must worry though that this validator loses delegations, and then stays in this list, but
	// isn't the highest quality....
	//
	// BUT, we provide a simple extrinsic for **anyone** to claim that a validator should be in this
	// list.
	#[pallet::storage]
	pub type TopValidators<T: Config> =
		StorageValue<Value = BoundedVec<ValidatorBacking<T>, ConstU32<100>>>;

	/// Pallets use events to inform users when important changes are made.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// We usually use passive tense for events.
		SomethingStored { something: u32, who: T::AccountId },
	}

	/// Errors inform users that something went wrong.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error
	#[pallet::error]
	pub enum Error<T> {
		TooManyValidators,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			// This is a pretty lightweight check that we do EVERY block, but then tells us when an
			// Epoch has passed...
			if n % T::EpochDuration::get() == BlockNumberFor::<T>::zero() {
				// CHANGE VALIDATORS LOGIC
				// You cannot return an error here, so you have to be clever with your code...
			}

			// We return a default weight because we do not expect you to do weights for your
			// project... Except for extra credit...
			return Weight::default();
		}
	}

	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsics", which are often compared to transactions.
	/// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#dispatchables
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example of directly updating the authorities into [`Config::ReportNewValidatorSet`].
		pub fn force_report_new_validators(
			origin: OriginFor<T>,
			new_set: Vec<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				(new_set.len() as u32) < T::MaxValidators::get(),
				Error::<T>::TooManyValidators
			);
			T::ReportNewValidatorSet::report_new_validator_set(new_set);
			Ok(())
		}

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_origin/index.html
			let who = ensure_signed(origin)?;

			// Do some checks.
			ensure!(something > 42, "ErrorsCanBeStaticStringsToo");

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });

			// Return a successful `DispatchResult`
			Ok(())
		}

		pub fn report_top_validator(origin: OriginFor<T>, _who: T::AccountId) -> DispatchResult {
			let _who_cares = ensure_signed(origin)?;
			//let validator_status = Validators::<T>::get(who);

			let _top_validators = TopValidators::<T>::get();

			// we assume top_validators is stored
			// let last_validator = top_validators.last();
			// ensure!(last_validator.backing < validator_status.backing);

			// top_validators.insert(new_validator).sort().truncate_to_100()

			// try to see if validator status is better than the last guy
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// A function to get you an account id for the current block author.
		pub fn find_author() -> Option<T::AccountId> {
			// If you want to see a realistic example of the `FindAuthor` interface, see
			// `pallet-authorship`.
			T::FindAuthor::find_author::<'_, Vec<_>>(Default::default())
		}
	}
}

pub trait DoSlash<T: Config> {
	fn do_slash(_who: T::AccountId, _amount: sp_runtime::Perbill);
}
impl<T: Config> DoSlash<T> for Pallet<T> {
	fn do_slash(_who: T::AccountId, _amount: sp_runtime::Perbill) {
		// lookup who
		// get all delegators
		// reduce who and delegators by amount
	}
}
