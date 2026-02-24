#![no_std]
use soroban_sdk::{contract, contractimpl, contracterror, symbol_short, Env, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    LimitExceeded = 1,
}

#[contract]
pub struct ErrorHandlingContract;

#[contractimpl]
impl ErrorHandlingContract {
    pub fn hello(env: Env, count: u32) -> Result<Symbol, Error> {
        if count > 10 {
            return Err(Error::LimitExceeded);
        }
        Ok(symbol_short!("Hello"))
    }
}

mod test;
