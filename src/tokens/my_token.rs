
use alloy_sol_types::sol;
use stylus_sdk::{
    alloy_primitives::{Address, U256}, msg, prelude::*,
    alloy_sol_types::SolError,
};

use crate::extensions::{erc20_burnable::Erc20Burnable, erc20_cap::Erc20Cap, erc20_pausable::Erc20Pausable};

use super::erc20::{Erc20, Erc20Params, Erc20Error};

pub struct MyTokenParams;

impl Erc20Params for MyTokenParams {
    const NAME: &'static str = "My test erc20 token";
    const SYMBOL: &'static str = "MT";
    const DECIMALS: u8 = 18;
}

sol_storage! {
    #[entrypoint]   // Makes MyToken the entrypoint
    pub struct MyToken {
        bool was_initialized;
        #[borrow]
        Erc20<MyTokenParams> erc20;
        #[borrow]
        Erc20Burnable erc20_burnable;
        #[borrow]
        Erc20Pausable erc20_pausable;
        #[borrow]
        Erc20Cap erc20_cap;
    }
}

sol! {
    error AlreadyInitialized();
}

pub enum MyTokenError {
    AlreadyInitialized(AlreadyInitialized),
}

impl From<MyTokenError> for Vec<u8> {
    fn from(e: MyTokenError) -> Vec<u8> {
        match e {
            MyTokenError::AlreadyInitialized(e) => e.encode(),
        }
    }
}

impl MyToken {

    /*** Erc20Pausable and Erc20Cap methods ***/

    /// override update(..) :
    ///   - function from Erc20 to only run when contract is not Paused
    ///   - checks if after balance update cap was not exceeded
    /// Notice:
    ///   - return error type is Vec<u8> because now update(..) can return errors from: Erc20Error | Erc20Pausable | Erc20Cap
    pub fn update(&mut self, from: Address, to: Address, value: U256) -> Result<(), Vec<u8>> {
        self.erc20_pausable.when_not_paused()?;

        self.erc20.update(from, to, value)?;

        self.erc20_cap.when_cap_not_exceeded(self.erc20.total_supply.get())?;
        Ok(())
    }
}

#[external]
#[inherit(Erc20<MyTokenParams>, Erc20Burnable, Erc20Pausable, Erc20Cap)]
impl MyToken {

    // constructor like function
    pub fn init(&mut self, cap: U256) -> Result<(), Vec<u8>> {
        
        self.erc20_cap.init_cap(cap)?;
        Ok(())
    }

    // for testing purposes, anyone can mint
    pub fn mint(&mut self, account: Address, amount: U256) -> Result<(), Erc20Error> {
        self.erc20.mint(account, amount)
    }

    /*** Erc20Burnable methods ***/

    pub fn burn(&mut self, amount: U256) -> Result<(), Erc20Error> {
        self.erc20.burn(msg::sender(), amount)
    }

    pub fn burn_from(&mut self, account: Address, amount: U256) -> Result<(), Erc20Error> {
        self.erc20.spend_allowance(account, msg::sender(), amount)?;
        self.erc20.burn(account, amount)
    }

}
