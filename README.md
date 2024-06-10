# PBA Assignment - POLKADOT-SDK / FRAME

## !! See [ASSIGNMENT.md](./ASSIGNMENT.md) for instructions to complete this assignment !!

---

## About This Template

This template is based on the `polkadot-sdk-minimal-template`. This is the most bare-bone template
that ships with `polkadot-sdk`, and most notably has no consensus mechanism. That is, any node in
the network can author blocks. This makes it possible to easily run a single-node network with any
block-time that we wish.

### ☯️ `omni-node`-only

Moreover, this template has been stripped to only contain the `runtime` part of the template. This
is because we provide you with an omni-node that can run this runtime. An `omni-node` is broadly a
substrate-based node that has no dependency to any given runtime, and can run a wide spectrum of
runtimes. The `omni-node` provided below is based on the aforementioned template and therefore has
no consensus engine baked into it.

## How to Run

### Individual Pallets

To test while developing, without a full build:

```sh
cargo t -p pallet-dpos
cargo t -p pallet-free-tx
cargo t -p pallet-multisig
cargo t -p pallet-treasury
```

### Entire Runtime

#### Using `omni-node`

First, make sure to install the special omni-node of the PBA assignment, if you have not done so
already from the previous activity.

```sh
cargo install --force --git https://github.com/kianenigma/pba-omni-node.git
```

Then, you have two options:

1. Run with the default genesis using the `--runtime` flag:

```sh
cargo build -p pba-runtime --release
pba-omni-node --runtime ./target/release/wbuild/pba-runtime/pba_runtime.wasm --tmp
```

2. Run with a more flexible genesis using the `--chain` flag:

```sh
cargo install staging-chain-spec-builder
```

Feel free to populate your chain-spec file then with more accounts, like:

```json
// under `balances.balance`
["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 100000],
["5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", 100000],
["5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y", 100000],
["5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy", 100000],
["5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw", 100000],
["5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL", 100000],
["5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY", 100000],
["5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc", 100000],
["5Ck5SLSHYac6WFt5UZRSsdJjwmpSZq85fd5TRNAdZQVzEAPT", 100000],
["5HKPmK9GYtE1PSLsS1qiYU9xQ9Si1NcEhdeCq9sw5bqu4ns8", 100000],
["5FCfAonRZgTFrTd9HREEyeJjDpT397KMzizE6T3DvebLFE7n", 100000],
["5CRmqmsiNFExV6VbdmPJViVxrWmkaXXvBrSX8oqBT8R9vmWk", 100000],
["5Fxune7f71ZbpP2FoY3mhYcmM596Erhv1gRue4nsPwkxMR4n", 100000],
["5CUjxa4wVKMj3FqKdqAUf7zcEMr4MYAjXeWmUf44B41neLmJ", 100000]
```

If you wish to set "Alice" as the sudo:

```json
// under `sudo.key`
"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
```

And more details like:

```json
"chainType": "Development"
"properties": {
  "tokenDecimals": 1,
  "tokenSymbol": "PBA"
}
```

Both of these patterns have already been explored in the FRAMELess activity, so you should be
familiar with them.
