pub mod mem;
pub use mem::MemActStore;

use crate::types::Account;

pub trait ActStore {
    fn get_account(&self, client: u16) -> Option<&Account>;
    fn deposit(&mut self, client: u16, amnt: u64) -> u64;
    fn withdraw(&mut self, client: u16, amnt: u64) -> u64;
    fn hold(&mut self, client: u16, amnt: u64) -> u64;
    fn unhold(&mut self, client: u16, amnt: u64) -> u64;
    fn lock_account(&mut self, client: u16) -> bool;
    fn unlock_account(&mut self, client: u16) -> bool;
}
