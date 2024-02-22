
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
};

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
        #[borrow] // inheritance is done with Rust composition plus Stylus magic
        Erc20<MyTokenParams> erc20;
    }
}

#[external]
#[inherit(Erc20<MyTokenParams>)]
impl MyToken {

    // for testing purposes, anyone can mint
    pub fn mint(&mut self, account: Address, amount: U256) -> Result<(), Erc20Error> {
        self.erc20.mint(account, amount)
    }

    // for testing purposes, anyone can burn
    pub fn burn(&mut self, account: Address, amount: U256) -> Result<(), Erc20Error> {
        self.erc20.burn(account, amount)
    }

}
