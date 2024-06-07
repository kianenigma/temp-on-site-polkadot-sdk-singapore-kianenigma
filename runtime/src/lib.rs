#![cfg_attr(not(feature = "std"), no_std)]

use frame::{
	deps::{
		codec::Compact,
		frame_support::{
			genesis_builder_helper::{build_state, get_preset},
			runtime,
			traits::AsEnsureOriginWithArg,
			weights::FixedFee,
		},
	},
	prelude::*,
	runtime::{
		apis::{
			self, impl_runtime_apis, ApplyExtrinsicResult, CheckInherentsResult,
			ExtrinsicInclusionMode, OpaqueMetadata,
		},
		prelude::*,
	},
	traits::{FindAuthor, One},
};
use pallet_transaction_payment::{ConstFeeMultiplier, FeeDetails, Multiplier, RuntimeDispatchInfo};

#[runtime_version]
const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("pba-assignment-template"),
	impl_name: create_runtime_str!("pba-assignment-template"),
	authoring_version: 1,
	spec_version: 0,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

// Composes the runtime by adding all the used pallets and deriving necessary types.
#[runtime]
mod runtime {
	/// The main runtime and the associated types that are used in the runtime.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask
	)]
	pub struct Runtime;

	/// Mandatory system pallet that should always be included in a FRAME runtime.
	#[runtime::pallet_index(0)]
	pub type System = frame_system;

	/// Provides the ability to keep track of balances.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_tokens/index.html
	#[runtime::pallet_index(1)]
	pub type Balances = pallet_balances;

	/// Provides a way to execute privileged functions.
	#[runtime::pallet_index(2)]
	pub type Sudo = pallet_sudo;

	/// Provides the ability to charge for extrinsic execution.
	#[runtime::pallet_index(3)]
	pub type TransactionPayment = pallet_transaction_payment;

	#[runtime::pallet_index(4)]
	pub type Assets = pallet_assets;

	#[runtime::pallet_index(5)]
	pub type Multisig = pallet_multisig;

	#[runtime::pallet_index(6)]
	pub type FreeTx = pallet_free_tx;

	#[runtime::pallet_index(7)]
	pub type Dpos = pallet_dpos;

	#[runtime::pallet_index(8)]
	pub type Treasury = pallet_treasury;

	#[runtime::pallet_index(99)]
	pub type Timestamp = pallet_timestamp;
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
}

// Note that the following `derive_impl` are using `TestDefaultConfig` for a number of crates. This
// is not recommended in production, but does make things easier in this context. You are advised to
// change these before launching a real chain.

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
	type Block = Block;
	type Version = Version;

	// Use the account data from the balances pallet
	type AccountData = pallet_balances::AccountData<Balance>;
	type Nonce = u32;

	// This is needed to make pjs-apps work and send txs
	type AccountId = frame::runtime::types_common::AccountId;
}

// Implements the types required for the balances pallet.
#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Runtime {
	// Store the accounts in the system pallet -- this is a common practice in FRAME-based runtimes.
	// See `pallet_balances::Account` for more information.
	type AccountStore = System;
	// set some other types.
	type Balance = u128;
	type ExistentialDeposit = ConstU128<10>;
}

#[derive_impl(pallet_sudo::config_preludes::TestDefaultConfig)]
impl pallet_sudo::Config for Runtime {}

parameter_types! {
	pub FeeMultiplier: Multiplier = Multiplier::one();
}

#[derive_impl(pallet_transaction_payment::config_preludes::TestDefaultConfig)]
impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, ()>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
	// These two define what the transaction fee would.
	type WeightToFee = FixedFee<5, Balance>;
	type LengthToFee = FixedFee<0, Balance>;
}

// we don't really need this pallet, but it PJS apps requires :(
#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig)]
impl pallet_timestamp::Config for Runtime {}

parameter_types! {
	pub const AssetDeposit: Balance = 100;
	pub const ApprovalDeposit: Balance = 1;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 10;
	pub const MetadataDepositPerByte: Balance = 1;
}

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u128;
	type AssetId = u32;
	type AssetIdParameter = Compact<u32>;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = ConstU128<1>;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type RemoveItemsLimit = ConstU32<1000>;
	type CallbackHandle = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

/// Configure the pallet-multisig in pallets/multisig.
impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type NativeBalance = Balances;
	type RuntimeCall = RuntimeCall;
}

/// Configure the pallet-free-tx in pallets/free-tx.
impl pallet_free_tx::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type NativeBalance = Balances;
	type RuntimeCall = RuntimeCall;
}

parameter_types! {
	pub const MaxValidators: u32 = 10;
}

pub struct BlockAuthor;
impl FindAuthor<AccountId> for BlockAuthor {
	fn find_author<'a, I>(_: I) -> Option<AccountId>
	where
		I: 'a + IntoIterator<Item = ([u8; 4], &'a [u8])>,
	{
		// return a random-ish block author previously reported.
		let last_known_set = ValidatorSet::get();
		if last_known_set.is_empty() {
			return None;
		}

		let index = frame_system::Pallet::<Runtime>::block_number() as usize % last_known_set.len();
		Some(last_known_set[index].clone())
	}
}

parameter_types! {
	// This is a temporary storage that will keep the validators. In reality, this would have been
	// `pallet-aura` or another pallet that would consume these.
	pub storage ValidatorSet: Vec<AccountId> = vec![];
}

pub struct StoreNewValidatorSet;
impl pallet_dpos::ReportNewValidatorSet<AccountId> for StoreNewValidatorSet {
	fn report_new_validator_set(new_set: Vec<AccountId>) {
		ValidatorSet::set(&new_set);
	}
}

/// Configure the pallet-dpos in pallets/dpos.
impl pallet_dpos::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type NativeBalance = Balances;
	type MaxValidators = MaxValidators;
	type FindAuthor = BlockAuthor;
	type ReportNewValidatorSet = StoreNewValidatorSet;
}

/// Configure the pallet-treasury in pallets/treasury.
impl pallet_treasury::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type NativeBalance = Balances;
	type Fungibles = Assets;
	type CustomOrigin = EnsureRoot<AccountId>;
}

/// The signed extensions that are added to the runtime.
type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

// Many of the types in this runtime are being pulled in from `derive_impl`. We use the almighty
// `<type as trait>::associated` to create local aliases to them.

type AccountId = <Runtime as frame_system::Config>::AccountId;
type Balance = <Runtime as pallet_balances::Config>::Balance;
type Nonce = <Runtime as frame_system::Config>::Nonce;
type Block = frame::runtime::types_common::BlockOf<Runtime, SignedExtra>;
type Header = HeaderFor<Runtime>;

type RuntimeExecutive =
	Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem>;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame::deps::frame_benchmarking::define_benchmarks!(
		[frame_benchmarking, BaselineBench::<Runtime>]
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_sudo, Sudo]
		[pallet_dpos, Dpos]
		[pallet_multisig, Multisig]
		[pallet_free_tx, FreeTx]
		[pallet_treasury, Treasury]
		[pallet_timestamp, Timestamp]
	);
}

impl_runtime_apis! {
	impl apis::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			RuntimeExecutive::execute_block(block)
		}

		fn initialize_block(header: &Header) -> ExtrinsicInclusionMode {
			RuntimeExecutive::initialize_block(header)
		}
	}

	impl apis::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl apis::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: ExtrinsicFor<Runtime>) -> ApplyExtrinsicResult {
			RuntimeExecutive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> HeaderFor<Runtime> {
			RuntimeExecutive::finalize_block()
		}

		fn inherent_extrinsics(data: InherentData) -> Vec<ExtrinsicFor<Runtime>> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: InherentData,
		) -> CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl apis::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: ExtrinsicFor<Runtime>,
			block_hash: <Runtime as frame_system::Config>::Hash,
		) -> TransactionValidity {
			RuntimeExecutive::validate_transaction(source, tx, block_hash)
		}
	}

	impl apis::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &HeaderFor<Runtime>) {
			RuntimeExecutive::offchain_worker(header)
		}
	}

	impl apis::SessionKeys<Block> for Runtime {
		fn generate_session_keys(_seed: Option<Vec<u8>>) -> Vec<u8> {
			Default::default()
		}

		fn decode_session_keys(
			_encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, apis::KeyTypeId)>> {
			Default::default()
		}
	}

	impl apis::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(uxt: ExtrinsicFor<Runtime>, len: u32) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(uxt: ExtrinsicFor<Runtime>, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame::deps::frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame::deps::frame_benchmarking::BenchmarkList>,
			Vec<frame::deps::frame_support::traits::StorageInfo>,
		) {
			use frame::deps::frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			use frame::deps::frame_support::traits::StorageInfoTrait;
			use frame::deps::frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame::deps::frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame::deps::frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame::deps::frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch};
			use frame::deps::sp_storage::TrackedStorageKey;
			use frame::deps::frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			impl frame::deps::frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			use frame::deps::frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame::deps::frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame::deps::frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = RuntimeExecutive::try_runtime_upgrade(checks).unwrap();
			(weight, <Runtime as frame_system::Config>::BlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame::deps::frame_try_runtime::TryStateSelect
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			RuntimeExecutive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, |_| None)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			vec![]
		}
	}
}
