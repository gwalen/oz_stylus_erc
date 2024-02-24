use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
};

use crate::tokens::erc20::{Erc20Error, FunctionNotImplemented};


sol_storage! {
    pub struct Erc20Burnable {}
}

#[external]
impl Erc20Burnable  {

    // due to error in Stylus (https://github.com/OffchainLabs/stylus-sdk-rs/issues/106)
    // this will be implemented in body of MyToken
    pub fn burn(&mut self, amount: U256) -> Result<(), Erc20Error> {
        Err(Erc20Error::FunctionNotImplemented(FunctionNotImplemented{}))
    }

    // due to error in Stylus (https://github.com/OffchainLabs/stylus-sdk-rs/issues/106)
    // this will be implemented in body of MyToken
    pub fn burn_from(&mut self, account: Address, amount: U256) -> Result<(), Erc20Error> {
        Err(Erc20Error::FunctionNotImplemented(FunctionNotImplemented{}))
    }
    
}
