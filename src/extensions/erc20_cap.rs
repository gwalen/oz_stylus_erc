use stylus_sdk::{
    alloy_primitives::U256,
    alloy_sol_types::{sol, SolError},
    prelude::*,
};


sol_storage! {
    pub struct Erc20Cap {
        uint256 cap;
    }
}

sol! {
    /**
     * @dev Total supply cap has been exceeded.
     */
    error ERC20ExceededCap(uint256 increasedSupply, uint256 cap);

    /**
     * @dev The supplied cap is not a valid cap.
     */
    error ERC20InvalidCap(uint256 cap);
}

pub enum Erc20CapError {
    ERC20ExceededCap(ERC20ExceededCap),    // 0x9e79f854
    ERC20InvalidCap(ERC20InvalidCap),      // 0x392e1e27
}

impl From<Erc20CapError> for Vec<u8> {
    fn from(e: Erc20CapError) -> Vec<u8> {
        match e {
            Erc20CapError::ERC20ExceededCap(e) => e.encode(),
            Erc20CapError::ERC20InvalidCap(e) => e.encode(),
        }
    }
}

impl Erc20Cap {

    pub fn set_cap(&mut self, cap: U256) -> Result<(), Erc20CapError> {
        if cap == U256::ZERO {
            return Err(Erc20CapError::ERC20InvalidCap(ERC20InvalidCap { cap }));
        }
        self.cap.set(cap);
        Ok(())
    }

    pub fn when_cap_not_exceeded(&self, total_supply: U256) -> Result<(), Erc20CapError> {
        if total_supply > self.cap.get() {
            return Err(Erc20CapError::ERC20ExceededCap(ERC20ExceededCap {
                increasedSupply: total_supply,
                cap: self.cap.get(),
            }));
        }
        Ok(())
    }
}

#[external]
impl Erc20Cap {}
