use alloc::{string::String, vec::Vec};
use core::marker::PhantomData;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::{sol, SolError},
    evm, msg,
    prelude::*,
};

use crate::tokens::{erc20::{Erc20, Erc20Error, Erc20Params}, my_token::MyTokenParams};

sol_storage! {
    pub struct Erc20Burnable {
        #[borrow]
        Erc20<MyTokenParams> erc20;
    }
}


#[external]
#[inherit(Erc20<MyTokenParams>)]
impl Erc20Burnable  {

    pub fn burn(&mut self, account: Address, amount: U256) -> Result<(), Erc20Error> {
        self.erc20.burn(account, amount)
    }

    pub fn balance_of_burn(&self, address: Address) -> Result<U256, Erc20Error> {
        Ok(self.erc20.balances.get(address))
    }

    pub fn diff(&self, address: Address, amount: U256) -> Result<U256, Erc20Error> {
        let balance = self.erc20.balances.get(address);
        let diff = U256::from(10000) + (balance - amount);
        Ok(diff)        
    }
}
