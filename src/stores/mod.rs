pub mod act_mem;
pub use act_mem::MemActStore;

use crate::types::Account;
use anyhow::Result;

pub trait ActStore {
    /// Trait to be implemented by account stores
    fn get_account(&self, client: u16) -> Option<&Account>;
    fn deposit(&mut self, client: u16, amnt: u64) -> Result<i64>;
    fn withdraw(&mut self, client: u16, amnt: u64) -> Result<i64>;
    fn withdraw_unchecked(&mut self, client: u16, amnt: u64) -> Result<i64>;
    fn hold(&mut self, client: u16, amnt: u64) -> Result<i64>;
    fn unhold(&mut self, client: u16, amnt: u64) -> Result<i64>;
    fn lock_account(&mut self, client: u16) -> bool;
    fn unlock_account(&mut self, client: u16) -> bool;
}
