use stylus_sdk::{
    alloy_sol_types::{sol, SolError}, evm, msg, prelude::*
};

sol_storage! {
    pub struct Erc20Pausable {
        bool paused;
    }
}

sol! {
    /// Emitted when the pause is triggered by `account`.
    event Paused(address account);

    /// Emitted when the pause is lifted by `account`.
    event Unpaused(address account);

    /// The operation failed because the contract is paused.
    error EnforcedPause();

    /// The operation failed because the contract is not paused.
    error ExpectedPause();
}

pub enum Erc20PausableError {
    EnforcedPause(EnforcedPause),
    ExpectedPause(ExpectedPause),
}

impl From<Erc20PausableError> for Vec<u8> {
    fn from(e: Erc20PausableError) -> Vec<u8> {
        match e {
            Erc20PausableError::EnforcedPause(e) => e.encode(),
            Erc20PausableError::ExpectedPause(e) => e.encode(),
        }
    }
}

impl Erc20Pausable {
    pub fn when_not_paused(&self) -> Result<(), Erc20PausableError> {
        if self.paused.get() {
            return Err(Erc20PausableError::EnforcedPause(EnforcedPause {}));
        }
        Ok(())
    }

    pub fn when_paused(&self) -> Result<(), Erc20PausableError> {
        if !self.paused.get() {
            return Err(Erc20PausableError::ExpectedPause(ExpectedPause {}));
        }
        Ok(())
    }
}

#[external]
impl Erc20Pausable {
    pub fn pause(&mut self) -> Result<(), Erc20PausableError> {
        self.paused.set(true);
        evm::log(Paused { account: msg::sender() });
        Ok(())
    }

    pub fn unpause(&mut self) -> Result<(), Erc20PausableError> {
        self.paused.set(false);
        evm::log(Unpaused { account: msg::sender() });
        Ok(())
    }
}
