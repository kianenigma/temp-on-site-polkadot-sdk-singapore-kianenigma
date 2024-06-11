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
	use crate::*;
	use frame_support::{
		dispatch::GetDispatchInfo,
		pallet_prelude::*,
		traits::{fungible, fungible::MutateHold, tokens::Precision},
	};
	use frame_system::{pallet_prelude::*, RawOrigin};
	use sp_runtime::traits::{Convert, Dispatchable};
	use sp_std::prelude::*;

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
			// You need to tell your trait bounds that the `Reason` is `RuntimeHoldReason`.
			+ fungible::hold::Mutate<Self::AccountId, Reason = Self::RuntimeHoldReason>
			+ fungible::freeze::Inspect<Self::AccountId>
			+ fungible::freeze::Mutate<Self::AccountId>;

		/// A type representing all calls available in your runtime.
		/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html
		type RuntimeCall: Parameter
			+ Dispatchable<
				RuntimeOrigin = Self::RuntimeOrigin,
				PostInfo = frame_support::dispatch::PostDispatchInfo,
			> + GetDispatchInfo;

		/// This allows a runtime developer to configure some constant "HoldAmount" balance, which
		/// can be used in the extrinsic logic.
		#[pallet::constant]
		type HoldAmount: Get<BalanceOf<Self>>;

		/// Overarching hold reason. Our `HoldReason` below will become a part of this "Outer Enum"
		/// thanks to the `#[runtime]` macro.
		type RuntimeHoldReason: From<HoldReason>;

		/// This is an example of converting one generic type into a different type
		/// by bringing the logic OUTSIDE of your pallet, and into the runtime
		/// where all types are concrete, and the runtime developer can directly implement
		/// the appropriate logic.
		///
		/// In this case, we convert the `Balance` type into the `Weight` type.
		type BalanceToWeightConverter: Convert<BalanceOf<Self>, Weight>;

		/// We use a configurable constant `BlockNumber` to tell us when we should trigger the
		/// free-tx era change.
		#[pallet::constant]
		type EraLength: Get<BlockNumberFor<Self>>;
	}

	/// A reason for the pallet placing a hold on funds.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// Funds are held to register for free transactions.
		#[codec(index = 0)]
		FreeTxHold,
	}

	/// The pallet's storage items.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#storage
	/// https://paritytech.github.io/polkadot-sdk/master/frame_support/pallet_macros/attr.storage.html
	#[pallet::storage]
	pub type Something<T> = StorageValue<Value = u32>;
	#[pallet::storage]
	pub type SomethingMap<T: Config> = StorageMap<Key = T::AccountId, Value = BlockNumberFor<T>>;

	// We use this storage to keep track of the balances held by our pallet. Generally a best
	// practice.
	#[pallet::storage]
	pub type AmountHeld<T: Config> = StorageMap<Key = T::AccountId, Value = BalanceOf<T>>;

	#[pallet::storage]
	pub type Credits<T: Config> = StorageMap<Key = T::AccountId, Value = Weight>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub initial_credits: Vec<(T::AccountId, Weight, BalanceOf<T>)>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			for (acc, cred, at_hold) in &self.initial_credits {
				Credits::<T>::insert(acc, cred);
				T::NativeBalance::hold(&HoldReason::FreeTxHold.into(), acc, *at_hold).unwrap();
				AmountHeld::<T>::insert(acc, at_hold);
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		#[cfg(feature = "try-runtime")]
		fn try_state(_: BlockNumberFor<T>) -> Result<(), sp_runtime::TryRuntimeError> {
			Self::do_try_state()
		}

		fn integrity_test() {
			assert!(!T::EraLength::get().is_zero(), "era length cannot be zero");
		}
	}

	#[cfg(any(test, feature = "try-runtime"))]
	impl<T: Config> Pallet<T> {
		pub(crate) fn do_try_state() {
			use sp_std::collections::btree_set::BTreeSet;
			let credits: BTreeSet<T::AccountId> = Credits::<T>::iter_keys().collect();
			let holds: BTreeSet<T::AccountId> = AmountHeld::<T>::iter_keys().collect();
			assert_eq!(credits, holds);
		}
	}

	/// Pallets use events to inform users when important changes are made.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// We usually use passive tense for events.
		TxSuccess,
	}

	/// Errors inform users that something went wrong.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		TxFailed,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// A user is trying to hold funds when they already have held funds.
		AlreadyHeld,
		/// A user is trying to release funds when they have no held funds.
		NothingHeld,
	}

	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsics", which are often compared to transactions.
	/// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#dispatchables
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that charges no fee if successful.
		pub fn free_tx(origin: OriginFor<T>, success: bool) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_origin/index.html
			let _who = ensure_signed(origin)?;

			// If this line fails, the user ends up paying a fee.
			ensure!(success, Error::<T>::TxFailed);

			// Deposit a basic event.
			Self::deposit_event(Event::<T>::TxSuccess);

			// This line tells the runtime to refund any fee taken, making the tx free.
			Ok(Pays::No.into())
		}

		/// An example of re-dispatching a call
		pub fn redispatch(
			origin: OriginFor<T>,
			call: Box<<T as Config>::RuntimeCall>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// The amount of weight credit the user has.
			let credit: Weight = Weight::default(); // TODO, actually store and get some weight credit
			if credit.all_gt(call.get_dispatch_info().weight) {
				// user has enough credit
			} else {
				// does not have enough credit
			}

			// Re-dispatch some call on behalf of the caller.
			let res = call.dispatch(RawOrigin::Signed(who).into());

			// Turn the result from the `dispatch` into our expected `DispatchResult` type.
			res.map(|_| ()).map_err(|e| e.error)
		}

		/// This function will hold the balance of some user, and track how much is held locally.
		pub fn hold_my_funds(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Only hold the funds of a user which has no holds already.
			ensure!(AmountHeld::<T>::get(&who).is_none(), Error::<T>::AlreadyHeld);
			T::NativeBalance::hold(&HoldReason::FreeTxHold.into(), &who, T::HoldAmount::get())?;
			// Store the amount held in our local storage.
			AmountHeld::<T>::insert(who, T::HoldAmount::get());
			Ok(())
		}

		/// This function will release the held balance of some user.
		pub fn release_my_funds(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Get the amount held by our system.
			// We call the `take` api so it also deletes the storage.
			let amount_held = AmountHeld::<T>::take(&who).ok_or(Error::<T>::NothingHeld)?;
			// NOTE: I am NOT using `T::HoldAmount::get()`... Why is that important?
			T::NativeBalance::release(
				&HoldReason::FreeTxHold.into(),
				&who,
				amount_held,
				Precision::BestEffort,
			)?;
			Ok(())
		}

		/// This function will release the held balance of some user.
		pub fn use_balance_to_weight(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			let _weight: Weight = T::BalanceToWeightConverter::convert(amount);
			// Do stuff with weight...
			Ok(())
		}

		// this is a useful dummy tx to create different calls with different weight amounts
		// the nicer way to do this is to put this in a new standalone pallet only used in your
		// test runtime
		#[pallet::weight({*_weight})]
		pub fn dummy_call_with_weight(_origin: OriginFor<T>, _weight: Weight) -> DispatchResult {
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the weight of a call.
		pub fn call_weight(call: <T as Config>::RuntimeCall) -> Weight {
			call.get_dispatch_info().weight
		}

		/// This is a function suggested by a student that wanted to know how to make such a
		/// function work. It is not expected that you need this function, or a function like it, in
		/// your project. It really depends on how you are designing your logic.
		/// So just use this as an example of how to construct and use a `Perbill` type.
		pub fn proportional_credit(credit: Weight) -> Weight {
			use sp_runtime::Perbill;

			let block_number: BlockNumberFor<T> = <frame_system::Pallet<T>>::block_number();
			let era_len = T::EraLength::get();
			let blocks_into_era: BlockNumberFor<T> = block_number % era_len;

			// This will compute a "percentage" from the fraction `blocks_into_era` / `era_len`
			let percentage_of_era: Perbill = Perbill::from_rational(blocks_into_era, era_len);

			// This multiplication is safe because `percentage_of_era` is always less than or equal
			// to one.
			let relative_credit = percentage_of_era * credit;

			relative_credit
		}
	}
}
