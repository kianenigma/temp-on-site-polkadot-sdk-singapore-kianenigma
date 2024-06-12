use crate::{self as pallet_multisig, MultisigType};
use frame_support::{
	derive_impl,
	traits::{ConstU128, ConstU16, ConstU32, ConstU64},
};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		Balances: pallet_balances,
		Multisig: pallet_multisig,
	}
);

// Feel free to remove more items from this, as they are the same as
// `frame_system::config_preludes::TestDefaultConfig`. We have only listed the full `type` list here
// for verbosity. Same for `pallet_balances::Config`.
// https://paritytech.github.io/polkadot-sdk/master/frame_support/attr.derive_impl.html
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ConstU32<10>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<10>;
}

impl pallet_multisig::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type NativeBalance = Balances;
	type RuntimeCall = RuntimeCall;

	fn multi_sig_filter(call: <Self as crate::Config>::RuntimeCall, m_type: MultisigType) -> bool {
		match m_type {
			MultisigType::All => true,
			MultisigType::TransferOnly => match call {
				RuntimeCall::Balances(_) => true,
				_ => false,
			},
			_ => false,
		}
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	// learn how to improve your test setup:
	// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

pub struct StateBuilder {
	pub num_accounts: u32,
	pub initial_balance: u32,
}

impl Default for StateBuilder {
	fn default() -> Self {
		Self { num_accounts: 5, initial_balance: 10 }
	}
}

#[allow(unused)]
impl StateBuilder {
	pub fn num_accounts(mut self, new_num_accounts: u32) -> Self {
		self.num_accounts = new_num_accounts;
		self
	}

	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		// build initial state based in self
		let mut ext: TestExternalities =
			frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into();
		ext.execute_with(|| {
			for _acc in 0..self.num_accounts {
				// create_account();
			}
		});

		// pre-all tests
		ext.execute_with(|| {
			test();
		});

		// post all tests
	}
}
