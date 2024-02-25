# ErcStylus 

Repository contains implementation of :
- `Erc20` token standard
- `Erc20Burnable` extension
- `Erc20Pausable` extension
- `Erc20Cap` extension
- `MyToken` contract - a concrete implementation of the Erc20 with all extensions

For each of those there is a set of tests that verify correctness of the implementation.

The tests are running against the Stylus testnet because setting up the whole local environment with local node to run tests would be cumbersome and long process. It was agreed with a team that it is not the scope of a task and that tests can be started against the testnet.

## Requirements for deploy and tests

Rust and stylus crates must be installed.

In order to run tests please create a `.env` file with following fields:
```
STYLUS_PROGRAM_ADDRESS=<address of the contract>
ALICE_PRIV_KEY_PATH="./.secret/alice_pk.key"
BOB_PRIV_KEY_PATH="./.secret/bob_pk.key"
RPC_URL=https://stylus-testnet.arbitrum.io/rpc
```
Also provide files with two private keys used for deployments and running tests. Those files should be located
in the `./secret` directory and named: `alice_pk.key` and `bob_pk.key`.

Current deployment address is : `0x8Ad82982059bD91ced51bd17359251b0EE65b077`


## Deploy

Deploy to Arbitrum Stylus testnet 
```
cargo stylus deploy \
  --private-key-path=./.secret/alice_pk.key
```  

Can verify the deployment on the block explorer : https://stylus-testnet-explorer.arbitrum.io/

## Test

Run tests :
```
cargo test --test erc20_base 
cargo test --test erc20_burnable 
cargo test --test erc20_pausable 
cargo test --test erc20_cap 
```

**Note:** 

Tests are missing checks for emitted events as current testing framework does not have a feature that would allow to check it.

## Design decisions and reasoning

The implementation structure was based on Solidity implementation of Erc20 standard created by Open Zeppelin.

During the implementation there was several workarounds that made a final solution less elegant but the had
to be made due to issues and missing features with current (`0.4.2`) stylus libs.

The issues where mostly about still not properly working emulation of Solidity inheritance. There is no inheritance in Rust and this needs to be emulated using special macros. As a result of that functions that 
should be overridden base on inheritance rules had to be move to implementation of `MyToken` adding 
unnecessary boilerplate code and redundancy. In future when Stylus framework will mature those issues should 
be fixed and code of this repo could be updated. 

Those issues were reported on team slack, each issue was also reported to Arbitrum Stylus github repo with example and explanation of the problem. 
Links to issues:
- [Storage access issues with multi-level inheritance](https://github.com/OffchainLabs/stylus-sdk-rs/issues/106)
- [Unable to declare non #[entrypoint] struct with #[borrow] and generics](https://github.com/OffchainLabs/stylus-sdk-rs/issues/107)
- [Function override does not work with inheritance as expected](https://github.com/OffchainLabs/stylus-sdk-rs/issues/109)


### Links to reference materials:

 - [Stylus docs](https://docs.arbitrum.io/stylus/stylus-gentle-introduction)
 - [Official Stylus github repo with examples](https://github.com/OffchainLabs/stylus-sdk-rs/tree/stylus)
 - [Stylus by example](https://arbitrum-stylus-by-example.vercel.app/basic_examples/constants)
 - [Awesome stylus links](https://github.com/OffchainLabs/awesome-stylus)
  - [OpenZeppelin Solidity sources for Erc20](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/ERC20.sol)
- Arbitrum Stylus channel on Arbitrum **Discord**