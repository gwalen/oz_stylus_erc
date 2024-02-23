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
    pub struct Erc20Burnable<T>  {
    // pub struct Erc20Burnable  {
        uint256 total_supply;
        #[borrow]
        Erc20<MyTokenParams> erc20;
        PhantomData<T> phantom;
    }
}

// impl<T> Erc20Burnable<T> where T: Erc20Params {
impl<T> Erc20Burnable<T> {
// impl Erc20Burnable {

    // pub fn burn_internal(erc20: &mut Erc20<T>, amount: U256) -> Result<(), Erc20Error> {
    //     erc20.burn(msg::sender(), amount)
    // }

    // TODO: check if it can be implmented by super class
    // pub fn erc20_get(&mut self) -> Result<&'static mut Erc20<MyTokenParams>, Erc20Error> {
    //     unimplemented!()
    // }
}

#[external]
#[inherit(Erc20<MyTokenParams>)]
// impl<T> Erc20Burnable<T> where T: Erc20Params + 'static {
impl<T> Erc20Burnable<T>  {
// impl Erc20Burnable {

    pub fn burn(&mut self, amount: U256) -> Result<(), Erc20Error> {
        self.erc20.burn(msg::sender(), amount)

        // self.erc20_get()?.burn(msg::sender(), amount)
    }
}
