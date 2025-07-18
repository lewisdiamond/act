use super::ActStore;
use crate::types::Account;
use anyhow::Result;
use std::collections::HashMap;
use std::collections::hash_map::IntoIter;

pub struct MemActStore(HashMap<u16, Account>);

enum Action {
    Withdraw(u64),
    Deposit(u64),
    Hold(u64),
    Unhold(u64),
}

impl MemActStore {
    // Creates an in-memory store for account data.
    // Useful for testing
    pub fn new() -> Self {
        MemActStore(HashMap::new())
    }

    fn action_act(&mut self, client: u16, action: Action) -> Result<i64> {
        let act = self.0.entry(client).or_insert_with(|| Account::new(client));
        match action {
            Action::Withdraw(amnt) => act.withdraw(amnt),
            Action::Deposit(amnt) => act.deposit(amnt),
            Action::Hold(amnt) => act.hold(amnt),
            Action::Unhold(amnt) => act.unhold(amnt),
        }
    }
}

impl Default for MemActStore {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for MemActStore {
    type Item = (u16, Account);

    type IntoIter = IntoIter<u16, Account>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl ActStore for MemActStore {
    fn deposit(&mut self, client: u16, amnt: u64) -> Result<i64> {
        self.action_act(client, Action::Deposit(amnt))
    }

    fn withdraw(&mut self, client: u16, amnt: u64) -> Result<i64> {
        self.action_act(client, Action::Withdraw(amnt))
    }

    fn withdraw_unchecked(&mut self, client: u16, amnt: u64) -> Result<i64> {
        if let Some(act) = self.0.get_mut(&client) {
            act.withdraw_allow_negative(amnt)
        } else {
            Err(anyhow::anyhow!("Account not found for client {}", client))
        }
    }

    fn hold(&mut self, client: u16, amnt: u64) -> Result<i64> {
        self.action_act(client, Action::Hold(amnt))
    }

    fn unhold(&mut self, client: u16, amnt: u64) -> Result<i64> {
        self.action_act(client, Action::Unhold(amnt))
    }

    fn lock_account(&mut self, client: u16) -> bool {
        if let Some(act) = self.0.get_mut(&client) {
            act.lock()
        } else {
            false
        }
    }

    fn unlock_account(&mut self, client: u16) -> bool {
        if let Some(act) = self.0.get_mut(&client) {
            act.unlock()
        } else {
            false
        }
    }

    fn get_account(&self, client: u16) -> Option<&Account> {
        self.0.get(&client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_balance() {
        let mut store = MemActStore::new();
        let client_id = 1;
        store.deposit(client_id, 200).expect("Deposit failed");

        let act = store
            .get_account(client_id)
            .expect("Could not get account from store");
        assert_eq!(client_id, act.id());
        assert!(matches!(act.available(), Ok(200)));
        let balance = store.deposit(client_id, 50);
        let act = store
            .get_account(client_id)
            .expect("Could not get account from store");
        assert_eq!(client_id, act.id());
        assert!(matches!(act.available(), Ok(250)));
        assert!(matches!(balance, Ok(250)));
    }

    #[test]
    fn test_sub_balance() {
        let mut store = MemActStore::new();
        let client_id = 1;
        store.deposit(client_id, 200).expect("Deposit failed");

        let act = store
            .get_account(client_id)
            .expect("Could not get account from store");
        assert_eq!(1, act.id());
        assert!(matches!(act.available(), Ok(200)));

        let balance = store.withdraw(client_id, 50).expect("Withdraw failed");
        let act = store
            .get_account(client_id)
            .expect("Could not get account from store");
        assert_eq!(1, act.id());
        assert!(matches!(act.available(), Ok(150)));
        assert_eq!(balance, 150);
    }

    #[test]
    fn test_negative_balance() {
        let mut store = MemActStore::new();
        let client_id = 1;
        store
            .withdraw(client_id, 1)
            .expect_err("Withdraw should fail");

        let act = store
            .get_account(client_id)
            .expect("Could not get account from store");

        assert_eq!(1, act.id());
        assert!(matches!(act.available(), Ok(0)));
    }

    #[test]
    fn test_hold() {
        let mut store = MemActStore::new();
        let client_id = 1;
        store.deposit(client_id, 100).expect("Deposit failed");

        let avail = store.hold(client_id, 10).expect("Hold failed");
        assert_eq!(90, avail);

        let act = store
            .get_account(client_id)
            .expect("Could not get account from store");

        assert_eq!(1, act.id());
        assert!(matches!(act.available(), Ok(90)));

        assert!(matches!(store.hold(client_id, 100), Ok(-10)));
        assert!(matches!(store.unhold(client_id, 20), Ok(10)));
    }

    #[test]
    fn test_lock() {
        let mut store = MemActStore::new();
        let client_id = 1;
        store.deposit(client_id, 100).expect("Deposit failed");
        let locked = store.lock_account(client_id);
        assert!(locked);

        let act = store
            .get_account(client_id)
            .expect("Could not get account from store");
        assert_eq!(1, act.id());
        assert!(act.is_locked());
    }
}
