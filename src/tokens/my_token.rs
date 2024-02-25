use alloy_sol_types::sol;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::SolError,
    msg,
    prelude::*,
};

use crate::extensions::{
    erc20_burnable::Erc20Burnable, erc20_cap::Erc20Cap, erc20_pausable::Erc20Pausable,
};

use super::erc20::{Erc20, Erc20Error, Erc20InvalidReceiver, Erc20InvalidSpender, Erc20Params};

pub struct MyTokenParams;

impl Erc20Params for MyTokenParams {
    const NAME: &'static str = "My test erc20 token";
    const SYMBOL: &'static str = "MT";
    const DECIMALS: u8 = 18;
}

sol_storage! {
    #[entrypoint]   // Makes MyToken the entrypoint
    pub struct MyToken {
        bool initialized;
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
    /*** Erc20Pausable ***/

    /// Definition of update() from Erc20 with additional functionalities
    /// Notice:
    ///  - this methods does not override update(..) from Erc20 - this is a bug is Stylus
    ///  - return error type is Vec<u8> because now update(..) can return errors from: Erc20Error | Erc20Pausable
    pub fn update(&mut self, from: Address, to: Address, value: U256) -> Result<(), Vec<u8>> {
        self.erc20_pausable.when_not_paused()?;
        self.erc20.update(from, to, value)?;
        Ok(())
    }
}

#[external]
#[inherit(Erc20<MyTokenParams>, Erc20Burnable, Erc20Pausable, Erc20Cap)]
impl MyToken {
    // constructor like function
    pub fn init(&mut self, cap: U256) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(MyTokenError::AlreadyInitialized(AlreadyInitialized {}).into());
        }
        self.erc20_cap.set_cap(cap)?;
        self.initialized.set(true);
        Ok(())
    }

    // we this to set cap on demand for testing
    pub fn set_cap(&mut self, cap: U256) -> Result<(), Vec<u8>> {
        self.erc20_cap.set_cap(cap)?;
        Ok(())
    }

    //TODO:
    // - add init() call to all tests !

    pub fn is_paused(&self) -> Result<bool, Erc20Error> {
        Ok(self.erc20_pausable.paused.get())
    }

    pub fn cap(&self) -> Result<U256, Erc20Error> {
        Ok(self.erc20_cap.cap.get())
    }

    pub fn total_supply(&self) -> Result<U256, Erc20Error> {
        Ok(self.erc20.total_supply.get())
    }

    /*** Erc20 methods manual override due to Stylus bug (109) ***/

    // for testing purposes, anyone can mint
    pub fn mint(&mut self, account: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.erc20_pausable.when_not_paused()?;
        self.erc20.mint(account, amount)?;
        self.erc20_cap.when_cap_not_exceeded(self.erc20.total_supply.get())?;
        Ok(())
    }

    pub fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Vec<u8>> {
        self.erc20_pausable.when_not_paused()?;
        let owner = msg::sender();
        self.transfer_internal(owner, to, value)?;
        Ok(true)
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        self.erc20_pausable.when_not_paused()?;
        let spender = msg::sender();
        self.erc20.spend_allowance(from, spender, value)?;
        self.transfer_internal(from, to, value)?;
        Ok(true)
    }

    fn transfer_internal(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        if from == Address::ZERO {
            return Err(Erc20Error::Erc20InvalidSpender(Erc20InvalidSpender {
                spender: Address::ZERO,
            })
            .into());
        }
        if to == Address::ZERO {
            return Err(Erc20Error::Erc20InvalidReceiver(Erc20InvalidReceiver {
                receiver: Address::ZERO,
            })
            .into());
        }

        self.update(from, to, value)
    }

    /*** Erc20Burnable methods ***/

    pub fn burn(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.erc20_pausable.when_not_paused()?;
        self.erc20.burn(msg::sender(), amount)?;
        Ok(())
    }

    pub fn burn_from(&mut self, account: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.erc20_pausable.when_not_paused()?;
        self.erc20.spend_allowance(account, msg::sender(), amount)?;
        self.erc20.burn(account, amount)?;
        Ok(())
    }
}
