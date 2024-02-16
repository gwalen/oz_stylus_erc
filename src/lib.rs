// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Initializes a custom, global allocator for Rust programs compiled to WASM.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

use alloc::{string::String, vec::Vec};
use core::{marker::PhantomData, panic};
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::{sol, SolError},
    evm, msg,
    prelude::*,
};

/// ERC20 base params
pub trait Erc20Params {
    /// token name
    const NAME: &'static str;
    /// token symbol
    const SYMBOL: &'static str;
    /// token decimals
    const DECIMALS: u8;
}

sol_storage! {
    /// ERC20 storage
    pub struct Erc20<T> {
        /// token balances
        mapping(address => uint256) balances;
        /// token allowances
        mapping(address => mapping(address => uint256)) allowances;
        /// total supply
        uint256 total_supply;
        /// special construct to allow having Erc20Params
        PhantomData<T> phantom;
    }
}

sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

     /// Indicates an error related to the current `balance` of a `sender`. Used in transfers.
     /// * `sender` - address whose tokens are being transferred.
     /// * `balance` - current balance for the interacting account.
     /// * `needed` - minimum amount required to perform a transfer.
    error Erc20InsufficientBalance(address sender, uint256 balance, uint256 needed);

    /// Indicates a failure with the `spender`'s `allowance`. Used in transfers.
    /// * `spender` - Address that may be allowed to operate on tokens without being their owner.
    /// * `allowance` - Amount of tokens a `spender` is allowed to operate with.
    /// * `needed` - Minimum amount required to perform a transfer.
    error Erc20InsufficientAllowance(address from, uint256 allowance, uint256 needed);
}

pub enum Erc20Error {
    Erc20InsufficientBalance(Erc20InsufficientBalance),
    Erc20InsufficientAllowance(Erc20InsufficientAllowance),
}

impl From<Erc20Error> for Vec<u8> {
    fn from(e: Erc20Error) -> Vec<u8> {
        match e {
            Erc20Error::Erc20InsufficientBalance(e) => e.encode(),
            Erc20Error::Erc20InsufficientAllowance(e) => e.encode(),
        }
    }
}

#[external]
impl<T: Erc20Params> Erc20<T> {
    pub fn name() -> Result<String, Erc20Error> {
        Ok(T::NAME.into())
    }

    pub fn symbol() -> Result<String, Erc20Error> {
        Ok(T::SYMBOL.into())
    }

    pub fn decimals() -> Result<u8, Erc20Error> {
        Ok(T::DECIMALS)
    }

    pub fn balance_of(&self, address: Address) -> Result<U256, Erc20Error> {
        todo!()
    }

    pub fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Erc20Error> {
        todo!()
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> Result<U256, Erc20Error> {
        todo!()
    }

    pub fn approve(&mut self, spender: Address, value: U256) -> Result<bool, Erc20Error> {
        todo!()
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Erc20Error> {
        todo!()
    }

    fn mint(&mut self, address: Address, value: U256) -> Result<(), Erc20Error> {
        todo!()
    }

    fn burn(&mut self, address: Address, value: U256) -> Result<(), Erc20Error> {
        todo!()
    }
}
