//! A shell pallet built with [`frame`].

#![cfg_attr(not(feature = "std"), no_std)]

// Re-export all pallet parts, this is needed to properly import the pallet into the runtime.
pub use pallet::*;

#[frame::pallet(dev_mode)]
pub mod pallet {
    use frame::prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        fn ed() -> Balance;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    pub type Balance = u128;

    #[pallet::storage]
    pub(crate) type TotalIssuance<T: Config> = StorageValue<_, Balance, ValueQuery>;

    #[pallet::storage]
    pub(crate) type Balances<T: Config> = StorageMap<Key = T::AccountId, Value = Balance>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        pub fn mint_unsafe(origin: T::RuntimeOrigin, amount: Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if amount < T::ed() {
                return Err("BelowEd".into());
            }
            // if who already has balance, we cannot mint again into it.
            if Balances::<T>::contains_key(&who) {
                return Err("AlreadyExists".into());
            }

            Balances::<T>::insert(&who, amount);

            let mut issuance = TotalIssuance::<T>::get();
            issuance += amount; // avoid unsafe math
            TotalIssuance::<T>::put(issuance);

            Ok(())
        }

        pub fn transfer(origin: T::RuntimeOrigin) -> DispatchResult {
            todo!()
        }
    }
}

#[cfg(test)]
mod test {
    use super::pallet as currency_pallet;
    use super::pallet::*;
    use frame::testing_prelude::*;

    construct_runtime! {
        pub enum Runtime {
            System: frame_system,
            Currency: currency_pallet,
        }
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl frame_system::Config for Runtime {
        type Block = MockBlock<Runtime>;

        // within pallet we just said `<T as frame_system::Config>::AccountId`, now we
        // finally specified it.
        type AccountId = u64;
    }

    impl currency_pallet::Config for Runtime {
        fn ed() -> Balance {
            5
        }
    }

    #[test]
    fn mint_works() {
        TestState::new_empty().execute_with(|| {
            // given
            assert_eq!(currency_pallet::Balances::<Runtime>::get(1), None);

            // when
            assert!(
                currency_pallet::Pallet::<Runtime>::mint_unsafe(RuntimeOrigin::signed(1), 42)
                    .is_ok()
            );

            // then
            assert_eq!(currency_pallet::Balances::<Runtime>::get(1), Some(42));
        })
    }

    #[test]
    fn mint_into_existing_fails() {
        todo!();
    }

    #[test]
    fn mint_below_ed_fails() {
        todo!();
    }
}
