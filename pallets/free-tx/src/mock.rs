use crate as pallet_free_tx;
use codec::Encode;
use frame_support::{
	derive_impl,
	dispatch::{DispatchResultWithPostInfo, GetDispatchInfo},
	pallet_prelude::Weight,
	traits::{ConstU128, ConstU16, ConstU32, ConstU64},
	weights::FixedFee,
};

use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, Convert, Dispatchable, IdentityLookup, SignedExtension},
	BuildStorage, DispatchError,
};

type Signature = ();
type SignedExtra = ();
#[allow(dead_code)]
type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
type BlockNumber = u64;
type Header = generic::Header<BlockNumber, BlakeTwo256>;
type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<AccountId, RuntimeCall, Signature, SignedExtra>;
type Block = generic::Block<Header, UncheckedExtrinsic>;
type Balance = u128;
type AccountId = u64;

// Configure a mock runtime to test the pallet. We use the simpler syntax here.
frame_support::construct_runtime! {
	pub struct Test {
		System: frame_system,
		Balances: pallet_balances,
		TransactionPayment: pallet_transaction_payment,
		FreeTx: pallet_free_tx,
	}
}

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
	type AccountId = AccountId;
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
	// Need to update the `RuntimeHoldReason` expected by the balances pallet.
	type RuntimeHoldReason = RuntimeHoldReason;
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<10>;
}

#[derive_impl(pallet_transaction_payment::config_preludes::TestDefaultConfig)]
impl pallet_transaction_payment::Config for Test {
	type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, ()>;
	type LengthToFee = FixedFee<1, Balance>;
	type WeightToFee = FixedFee<0, Balance>;
}

// In the runtime, we can actually implement logic to convert balance to weight, because
// WE KNOW that balance is u128, and Weight is two `u64`.
pub struct SimpleBalanceToWeight;
impl Convert<Balance, Weight> for SimpleBalanceToWeight {
	fn convert(input: Balance) -> Weight {
		use sp_runtime::SaturatedConversion;
		let saturated_u64: u64 = input.saturated_into();
		return Weight::from_parts(saturated_u64, saturated_u64);
	}
}

impl pallet_free_tx::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type NativeBalance = Balances;
	type RuntimeCall = RuntimeCall;
	/// Here we specify that `HoldAmount` is 1337. Anyone using your pallet can implement their own
	/// amount.
	type HoldAmount = ConstU128<1337>;
	/// Here we pull the `RuntimeHoldReason` generated by the `#[runtime]` macro.
	type RuntimeHoldReason = RuntimeHoldReason;
	/// Use our simple converter...
	type BalanceToWeightConverter = SimpleBalanceToWeight;
	// Assuming blocks happen every 6 seconds, this will be 600 seconds, approximately 10 minutes.
	// But this is all just test config, but gives you an idea how this is all CONFIGURABLE
	type EraLength = ConstU64<100>;
}

pub struct StateBuilder {
	pub initial_credits: Vec<(AccountId, Weight, Balance)>,
	pub initial_balances: Vec<(AccountId, Balance)>,
}

impl Default for StateBuilder {
	fn default() -> Self {
		Self {
			initial_balances: vec![(1, 10), (2, 10)],
			initial_credits: vec![(1, Weight::from_parts(10, 10), 5)],
		}
	}
}

impl StateBuilder {
	pub(crate) fn with_balance(mut self, acc: AccountId, balance: Balance) -> Self {
		self.initial_balances.push((acc, balance));
		self
	}

	pub(crate) fn build_and_execute(self, test: impl FnOnce() -> ()) {
		let system = frame_system::GenesisConfig::<Test>::default();
		let balances = pallet_balances::GenesisConfig::<Test> { balances: self.initial_balances };
		let free_tx =
			pallet_free_tx::GenesisConfig::<Test> { initial_credits: self.initial_credits };

		let mut ext: TestExternalities =
			RuntimeGenesisConfig { system, balances, free_tx, ..Default::default() }
				.build_storage()
				.unwrap()
				.into();

		ext.execute_with(|| {
			test();
			FreeTx::do_try_state();
		});
	}
}

pub(crate) fn dispatch_with_signed_exts(
	who: AccountId,
	call: RuntimeCall,
) -> DispatchResultWithPostInfo {
	let info = call.get_dispatch_info();
	let len = call.encode().len();

	let pre = pallet_transaction_payment::ChargeTransactionPayment::<Test>::from(0)
		.pre_dispatch(&who, &call, &info, len)
		.unwrap();

	let dispatch_result: Result<frame_support::dispatch::PostDispatchInfo, _> =
		call.dispatch(RuntimeOrigin::signed(who));

	let _post_dispatch_result =
		pallet_transaction_payment::ChargeTransactionPayment::<Test>::post_dispatch(
			Some(pre),
			&info,
			&dispatch_result.clone().unwrap(),
			len,
			&dispatch_result
				.clone()
				.map(|_| ())
				.map_err(|_| DispatchError::Other("DONT_CARE")),
		);

	dispatch_result
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	// learn how to improve your test setup:
	// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

#[allow(dead_code)]
pub fn mock_extrinsic(account_id: AccountId, call: RuntimeCall) -> UncheckedExtrinsic {
	let extra: SignedExtra = (
		// TODO: Add whatever signed extensions you want, such as:
		// pallet_transaction_payment::ChargeTransactionPayment::<Test>::from(0),
	);
	let raw_payload = SignedPayload::new(call, extra).unwrap();
	let (call, extra, _) = raw_payload.deconstruct();
	UncheckedExtrinsic::new_signed(call, account_id.into(), (), extra)
}
