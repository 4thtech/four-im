<p align="center">
  <a href="https://solana.com">
    <img alt="Solana" src="https://i.imgur.com/uBVzyX3.png" width="250" />
  </a>
</p>

# E-delivery on Solana

## Quick Start

### Configure CLI

> If you're on Windows, it is recommended to use [WSL](https://docs.microsoft.com/en-us/windows/wsl/install-win10) to run these commands

1. Set CLI config url to localhost cluster

```bash
solana config set --url localhost
```

2. Create CLI Keypair

If this is your first time using the Solana CLI, you will need to generate a new keypair:

```bash
solana-keygen new
```

### Start local Solana cluster

This example connects to a local Solana cluster by default.

Start a local Solana cluster:
```bash
solana-test-validator
```
> **Note**: You may need to do some [system tuning](https://docs.solana.com/running-validator/validator-start#system-tuning) (and restart your computer) to get the validator to run

Listen to transaction logs:
```bash
solana logs
```

### Install npm dependencies

```bash
npm install
```

### Build the on-chain program

There is both a Rust and C version of the on-chain program, whichever is built
last will be the one used when running the example.

```bash
npm run build:program
```

### Deploy the on-chain program

```bash
solana program deploy dist/program/instant_messaging.so
```

### Run the JavaScript client

```bash
npm run start
```

### Expected output

Public key values will differ:

```bash
Let's say hello to a Solana account...
Connection to cluster established: http://localhost:8899 { 'feature-set': 2045430982, 'solana-core': '1.7.8' }
Using account AiT1QgeYaK86Lf9kudqKthQPCWwpG8vFA1bAAioBoF4X containing 0.00141872 SOL to pay for fees
Using program Dro9uk45fxMcKWGb1eWALujbTssh6DW8mb4x8x3Eq5h6
Creating account 8MBmHtJvxpKdYhdw6yPpedp6X6y2U9dCpdYaZJdmwV3A to say hello to
Saying hello to 8MBmHtJvxpKdYhdw6yPpedp6X6y2U9dCpdYaZJdmwV3A
8MBmHtJvxpKdYhdw6yPpedp6X6y2U9dCpdYaZJdmwV3A has been greeted 1 times
Success
```

#### Not seeing the expected output?

- Ensure you've [started the local cluster](#start-local-solana-cluster),
  [built the on-chain program](#build-the-on-chain-program) and [deployed the program to the cluster](#deploy-the-on-chain-program).
- Inspect the program logs by running `solana logs` to see why the program failed.
  - ```bash
    Transaction executed in slot 5621:
    Signature: 4pya5iyvNfAZj9sVWHzByrxdKB84uA5sCxLceBwr9UyuETX2QwnKg56MgBKWSM4breVRzHmpb1EZQXFPPmJnEtsJ
    Status: Error processing Instruction 0: Program failed to complete
    Log Messages:
      Program G5bbS1ipWzqQhekkiCLn6u7Y1jJdnGK85ceSYLx2kKbA invoke [1]
      Program log: Hello World Rust program entrypoint
      Program G5bbS1ipWzqQhekkiCLn6u7Y1jJdnGK85ceSYLx2kKbA consumed 200000 of 200000 compute units
      Program failed to complete: exceeded maximum number of instructions allowed (200000) at instruction #334
      Program G5bbS1ipWzqQhekkiCLn6u7Y1jJdnGK85ceSYLx2kKbA failed: Program failed to complete

### Customizing the Program

To customize the example, make changes to the files under `/src`.  If you change
any files under `/src/program` you will need to
[rebuild the on-chain program](#build-the-on-chain-program) and [redeploy the program](#deploy-the-on-chain-program).

Now when you rerun `npm run start`, you should see the results of your changes.

## Learn about Solana

More information about how Solana works is available in the [Solana
documentation](https://docs.solana.com/) and all the source code is available on
[github](https://github.com/solana-labs/solana)

Further questions? Visit us on [Discord](https://discordapp.com/invite/pquxPsq)

## Learn about the client

The client in this example is written in TypeScript using:
- [Solana web3.js SDK](https://github.com/solana-labs/solana-web3.js)
- [Solana web3 API](https://solana-labs.github.io/solana-web3.js)

### Entrypoint

The [client's
entrypoint](https://github.com/solana-labs/example-helloworld/blob/ad52dc719cdc96d45ad8e308e8759abf4792b667/src/client/main.ts#L13)
does five things.

### Establish a connection to the cluster

The client establishes a connection with the cluster by calling
[`establishConnection`](https://github.com/solana-labs/example-helloworld/blob/ad52dc719cdc96d45ad8e308e8759abf4792b667/src/client/hello_world.ts#L92).

### Establish an account to pay for transactions

The client ensures there is an account available to pay for transactions,
and creates one if there is not, by calling
[`establishPayer`](https://github.com/solana-labs/example-helloworld/blob/ad52dc719cdc96d45ad8e308e8759abf4792b667/src/client/hello_world.ts#L102).

### Check if the instant_messaging on-chain program has been deployed

In [`checkProgram`](https://github.com/solana-labs/example-helloworld/blob/ad52dc719cdc96d45ad8e308e8759abf4792b667/src/client/hello_world.ts#L144),
the client loads the keypair of the deployed program from `./dist/program/instant_messaging-keypair.json` and uses
the public key for the keypair to fetch the program account. If the program doesn't exist, the client halts
with an error. If the program does exist, it will create a new account with the program assigned as its owner
to store program state (number of hello's processed).

## Learn about the on-chain program

The [on-chain helloworld program](/src/program-rust/Cargo.toml) is a Rust program
compiled to [Berkley Packet Format
(BPF)](https://en.wikipedia.org/wiki/Berkeley_Packet_Filter) and stored as an
[Executable and Linkable Format (ELF) shared
object](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format).

The program is written using:
- [Solana Rust SDK](https://github.com/solana-labs/solana/tree/master/sdk)

### Programming on Solana

To learn more about Solana programming model refer to the [Programming Model
Overview](https://docs.solana.com/developing/programming-model/overview).

To learn more about developing programs on Solana refer to the [On-Chain 
Programs Overview](https://docs.solana.com/developing/on-chain-programs/overview)

## Pointing to a public Solana cluster

Solana maintains three public clusters:
- `devnet` - Development cluster with airdrops enabled
- `testnet` - Tour De Sol test cluster without airdrops enabled
- `mainnet-beta` -  Main cluster

Use the Solana CLI to configure which cluster to connect to.

To point to `devnet`:
```bash
solana config set --url devnet
```

To point back to the local cluster:
```bash
solana config set --url localhost
```

## Expand your skills with advanced examples

There is lots more to learn; The following examples demonstrate more advanced
features like custom errors, advanced account handling, suggestions for data
serialization, benchmarking, etc...

- [Programming
  Examples](https://github.com/solana-labs/solana-program-library/tree/master/examples)
- [Token
  Program](https://github.com/solana-labs/solana-program-library/tree/master/token)
- [Token Swap
  Program](https://github.com/solana-labs/solana-program-library/tree/master/token-swap)
