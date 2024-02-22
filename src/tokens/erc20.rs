use alloc::{string::String, vec::Vec};
use core::marker::PhantomData;
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
    /// * `spender` - address that may be allowed to operate on tokens without being their owner.
    /// * `allowance` - amount of tokens a `spender` is allowed to operate with.
    /// * `needed` - minimum amount required to perform a transfer.
    error Erc20InsufficientAllowance(address sender, uint256 allowance, uint256 needed);

    /// Indicates a failure with the `approver` of a token to be approved. Used in approvals.
    /// * `approver` - address initiating an approval operation.
    error Erc20InvalidApprover(address approver);

    /// Indicates a failure with the `spender` to be approved. Used in approvals.
    /// * `spender` - address that may be allowed to operate on tokens without being their owner.
    error Erc20InvalidSpender(address spender);

    /// Indicates a failure with the token `receiver`. Used in transfers.
    /// * `receiver` - address to which tokens are being transferred.
    error Erc20InvalidReceiver(address receiver);
}

pub enum Erc20Error {
    Erc20InsufficientBalance(Erc20InsufficientBalance),
    Erc20InsufficientAllowance(Erc20InsufficientAllowance),
    Erc20InvalidSpender(Erc20InvalidSpender),
    Erc20InvalidApprover(Erc20InvalidApprover),
    Erc20InvalidReceiver(Erc20InvalidReceiver),
}

impl From<Erc20Error> for Vec<u8> {
    fn from(e: Erc20Error) -> Vec<u8> {
        match e {
            Erc20Error::Erc20InsufficientBalance(e) => e.encode(),
            Erc20Error::Erc20InsufficientAllowance(e) => e.encode(),
            Erc20Error::Erc20InvalidSpender(e) => e.encode(),
            Erc20Error::Erc20InvalidApprover(e) => e.encode(),
            Erc20Error::Erc20InvalidReceiver(e) => e.encode(),
        }
    }
}

/// Methods in this file are not exposed to other contracts (for that they must be under #[external] macro).
/// If you want other contracts to be able to "extend" your contract and be able to "inherit" some methods that are not external you must put them here and make 
/// public, in this way they will be visible by Rust in other structs that want to call them.
impl<T: Erc20Params> Erc20<T> {

    /// Creates a `value` amount of tokens and assigns them to `account`, by transferring it from address(0).
    /// Relies on the `_update` mechanism
    ///
    /// Emits a {Transfer} event with `from` set to the zero address.
    ///
    /// NOTE: This function is not virtual, {_update} should be overridden instead.
    pub fn mint(&mut self, account: Address, value: U256) -> Result<(), Erc20Error> {
        if account == Address::ZERO {
            return Err(Erc20Error::Erc20InvalidReceiver(Erc20InvalidReceiver {
                receiver: Address::ZERO,
            }));
        }
        self.update(Address::ZERO, account, value)
    }

    /// Destroys a `value` amount of tokens from `account`, lowering the total supply.
    /// Relies on the `_update` mechanism.
    ///
    /// Emits a {Transfer} event with `to` set to the zero address.
    ///
    /// NOTE: This function is not virtual, {_update} should be overridden instead.
    pub fn burn(&mut self, account: Address, value: U256) -> Result<(), Erc20Error> {
        if account == Address::ZERO {
            return Err(Erc20Error::Erc20InvalidSpender(Erc20InvalidSpender {
                spender: Address::ZERO,
            }));
        }
        self.update(account, Address::ZERO, value)
    }

    /// Transfers a `value` amount of tokens from `from` to `to`, or alternatively mints (or burns) if `from`
    /// (or `to`) is the zero address. All customizations to transfers, mints, and burns should be done by overriding
    /// this function.
    ///
    /// Emits a {Transfer} event.
    pub fn update(&mut self, from: Address, to: Address, value: U256) -> Result<(), Erc20Error> {
        if from == Address::ZERO {  // mint
            let total_supply = self.total_supply.get();
            self.total_supply.set(total_supply + value);
        } else {
            let mut from_balance_ref = self.balances.setter(from);
            let from_balance_value = from_balance_ref.get();
            if from_balance_value < value {
                return  Err(Erc20Error::Erc20InsufficientBalance(Erc20InsufficientBalance {
                    sender: from,
                    balance: from_balance_value,
                    needed: value,
                }));
            }
            // Overflow not possible: value <= fromBalance <= totalSupply.
            from_balance_ref.set(from_balance_value - value);
        }
        
        if to == Address::ZERO {  // burn
            // Overflow not possible: value <= totalSupply or value <= fromBalance <= totalSupply.
            let total_supply = self.total_supply.get();
            self.total_supply.set(total_supply - value);
        } else {
            let mut to_balance_ref = self.balances.setter(to);
            let to_balance_value = to_balance_ref.get();
            // Overflow not possible: balance + value is at most totalSupply, which we know fits into a uint256.
            to_balance_ref.set(to_balance_value + value);
        }

        evm::log(Transfer { from, to, value });
        Ok(())
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
        Ok(self.balances.get(address))
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> Result<U256, Erc20Error> {
        Ok(self.allowances.get(owner).get(spender))
    }

    /// Sets a `value` amount of tokens as the allowance of `spender` over the
    /// caller's tokens.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// IMPORTANT: Beware that changing an allowance with this method brings the risk
    /// that someone may use both the old and the new allowance by unfortunate
    /// transaction ordering. One possible solution to mitigate this race
    /// condition is to first reduce the spender's allowance to 0 and set the
    /// desired value afterwards:
    /// https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
    /// 
    /// * NOTE: If `value` is the maximum `uint256`, the allowance is not updated on
    ///         `transferFrom`. This is semantically equivalent to an infinite approval.
    ///
    /// Emits an {Approval} event.
    pub fn approve(&mut self, spender: Address, value: U256) -> Result<bool, Erc20Error> {
        let owner = msg::sender();
        self.approve_internal(owner, spender, value)?;
        Ok(true)
    }

    /// Moves a `value` amount of tokens from the caller's account to `to`.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// Emits a {Transfer} event.
    pub fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Erc20Error> {
        let owner = msg::sender();
        self.transfer_internal(owner, to, value)?;
        Ok(true)
    }

    /// Moves a `value` amount of tokens from `from` to `to` using the
    /// allowance mechanism. `value` is then deducted from the caller's
    /// allowance.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    /// 
    /// NOTE: Does not update the allowance if the current allowance is the maximum `uint256`.
    ///
    /// Emits a  {Transfer} event.
    /// Emits an {Approval} event indicating the updated allowance (this is not required by the ERC)
    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Erc20Error> {
        let spender = msg::sender();
        self.spend_allowance(from, spender, value)?;
        self.transfer_internal(from, to, value)?;
        Ok(true)
    }

    fn transfer_internal(&mut self, from: Address, to: Address, value: U256) -> Result<(), Erc20Error> {
        if from == Address::ZERO {
            return Err(Erc20Error::Erc20InvalidSpender(Erc20InvalidSpender {
                spender: Address::ZERO,
            }));    
        }
        if to == Address::ZERO {
            return Err(Erc20Error::Erc20InvalidReceiver(Erc20InvalidReceiver {
                receiver: Address::ZERO,
            }));
        }

        self.update(from, to, value)
    }

    

    fn approve_internal(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Erc20Error> {
        self.approve_internal_conditional(owner, spender, value, true)
    }

    fn approve_internal_conditional(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
        emit_event: bool,
    ) -> Result<(), Erc20Error> {
        if owner == Address::ZERO {
            return Err(Erc20Error::Erc20InvalidApprover(Erc20InvalidApprover {
                approver: Address::ZERO,
            }));
        }
        if spender == Address::ZERO {
            return Err(Erc20Error::Erc20InvalidSpender(Erc20InvalidSpender {
                spender: Address::ZERO,
            }));
        }

        self.allowances.setter(owner).insert(spender, value);

        if emit_event {
            evm::log(Approval {
                owner,
                spender,
                value,
            });
        }
        Ok(())
    }

    fn spend_allowance(&mut self, owner: Address, spender: Address, value: U256) -> Result<(), Erc20Error> {
        let current_allowance = self.allowances.get(owner).get(spender);
        if current_allowance != U256::MAX {
            if current_allowance < value {
                return Err(Erc20Error::Erc20InsufficientAllowance(Erc20InsufficientAllowance {
                    sender: owner,
                    allowance: current_allowance,
                    needed: value,
                }));
            }
            self.approve_internal_conditional(owner, spender, current_allowance - value, false)?;
        }
        Ok(())
    }
}
