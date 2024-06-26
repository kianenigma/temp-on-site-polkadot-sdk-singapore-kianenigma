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
		traits::{fungible, fungibles},
	};
	use frame_system::pallet_prelude::*;

	pub type AssetIdOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::AssetId;

	pub type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	pub type AssetBalanceOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
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

		/// Type to access the Assets Pallet.
		type Fungibles: fungibles::Inspect<Self::AccountId>
			+ fungibles::Mutate<Self::AccountId>
			+ fungibles::Create<Self::AccountId>;

		// two ways to convert asset and balance type to one another, look into `ConvertBack` for
		// reverse conversion, or define a second type.
		// type AssetIdToBalance: Convert<AssetBalanceOf<Self>, BalanceOf<Self>>;
		// fn asset_id_to_balance(id: AssetBalanceOf<Self>) -> BalanceOf<Self>;
		// or, do something like this:
		// type Fungibles: fungibles::Inspect<Self::AccountId, Balance = BalanceOf<Self>>

		// A custom, configurable origin that you can use. It can be wired to be `EnsureSigned`,
		// `EnsureRoot`, or any custom implementation at the runtime level.
		// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_origin/index.html#asserting-on-a-custom-external-origin
		type CustomOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		// Here is an associated type to give you access to your simple asset price lookup function
		type AssetPriceLookup: crate::AssetPriceLookup<Self>;

		// Small Spender
		type SmallSpender: EnsureOrigin<Self::RuntimeOrigin>;
	}

	/// The pallet's storage items.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#storage
	/// https://paritytech.github.io/polkadot-sdk/master/frame_support/pallet_macros/attr.storage.html
	#[pallet::storage]
	pub type Something<T> = StorageValue<Value = u32>;
	#[pallet::storage]
	pub type SomethingMap<T: Config> = StorageMap<Key = T::AccountId, Value = BlockNumberFor<T>>;

	/// Errors inform users that something went wrong.
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
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Here is an example of explicitly telling SCALE codec to encode a number as compact in
	// storage.
	#[derive(TypeInfo, Encode, Decode, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct StoreACompactNumber<T: Config> {
		who: T::AccountId,
		#[codec(compact)]
		amount: BalanceOf<T>,
	}

	#[pallet::storage]
	pub type MyCompactNumber<T: Config> = StorageValue<Value = StoreACompactNumber<T>>;

	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsics", which are often compared to transactions.
	/// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#dispatchables
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// // https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_origin/index.html
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });

			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		pub fn propose_spend(
			origin: OriginFor<T>,
			who: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			T::SmallSpender::ensure_origin(origin)?;
			Self::do_propose_spend(who, amount)
		}

		// Let's imagine you wanted to build a transfer extrinsic inside your pallet...
		// This doesn't really make sense to do, since this functionality exists in the `Balances`
		// pallet, but it illustrates how to use the `BalanceOf<T>` type and the `T::NativeBalance`
		// api.
		pub fn my_transfer_function(
			origin: OriginFor<T>,
			to: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			// Probably you would import these at the top of you file, not here, but just trying to
			// illustrate that you need to import this trait to access the function inside of it.
			use frame_support::traits::fungible::Mutate;
			// Read the docs on this to understand what it does...
			use frame_support::traits::tokens::Preservation;

			let sender = ensure_signed(origin)?;
			T::NativeBalance::transfer(&sender, &to, amount, Preservation::Expendable)?;
			Ok(())
		}
	}

	// NOTE: This is regular rust, and how you would implement functions onto an object.
	// These functions are NOT extrinsics, are NOT callable by outside users, and are really
	// just internal functions.
	//
	// Compare this to the block above which has `#[pallet::call]` which makes them extrinsics!
	impl<T: Config> Pallet<T> {
		fn do_propose_spend(_who: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			// Write the logic for your extrinsic here, since this is "outside" of the macros.
			// Following this kind of best practice can even allow you to move most of your
			// pallet logic into different files, with better, more clear structure, rather
			// than having a single huge complicated file.
			if amount > 100_000u32.into() {
				return Err("you are spending too much  money!".into());
			}

			Ok(())
		}
	}
}

/// This is some simple function that can be used to convert some amount of Asset A, and turn it
/// into some amount of Asset B. You do not really need to implement this function. You would to
/// build your own complex pallet to figure this out (oracle, dex) BUT you can make the assumption
/// that somewhere this logic exists, and then you can use it.
pub trait AssetPriceLookup<T: Config> {
	fn price_lookup(
		asset_a_id: AssetIdOf<T>,
		asset_a_amount: AssetBalanceOf<T>,
		asset_b_id: AssetIdOf<T>,
	) -> AssetBalanceOf<T>;
}
