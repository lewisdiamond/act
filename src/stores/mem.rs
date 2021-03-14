use super::ActStore;
use crate::types::Account;
use std::collections::hash_map::IntoIter;
use std::collections::HashMap;

pub struct MemActStore(HashMap<u16, Account>);

enum Action {
    Withdraw(u64),
    Deposit(u64),
    Hold(u64),
    Unhold(u64),
}

impl MemActStore {
    pub fn new() -> Self {
        MemActStore(HashMap::new())
    }

    fn action_act(&mut self, client: u16, action: Action) -> u64 {
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
    fn deposit(&mut self, client: u16, amnt: u64) -> u64 {
        self.action_act(client, Action::Deposit(amnt))
    }

    fn withdraw(&mut self, client: u16, amnt: u64) -> u64 {
        self.action_act(client, Action::Withdraw(amnt))
    }

    fn hold(&mut self, client: u16, amnt: u64) -> u64 {
        self.action_act(client, Action::Hold(amnt))
    }

    fn unhold(&mut self, client: u16, amnt: u64) -> u64 {
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
        let client_id = 200;
        store.deposit(client_id, 200);

        if let Some(act) = store.get_account(client_id) {
            assert_eq!(200, act.id());
            assert_eq!(200, act.available());
        } else {
            panic!("Could not get account from store");
        }
        let balance = store.deposit(client_id, 50);
        if let Some(act) = store.get_account(client_id) {
            assert_eq!(200, act.id());
            assert_eq!(250, act.available());
            assert_eq!(250, balance);
        } else {
            panic!("Could not get account from store");
        }
    }

    #[test]
    fn test_sub_balance() {
        let mut store = MemActStore::new();
        let client_id = 1;
        store.deposit(client_id, 200);

        if let Some(act) = store.get_account(client_id) {
            assert_eq!(1, act.id());
            assert_eq!(200, act.available());
        } else {
            panic!("Could not get account from store");
        }
        let balance = store.withdraw(client_id, 50);
        if let Some(act) = store.get_account(client_id) {
            assert_eq!(1, act.id());
            assert_eq!(150, act.available());
            assert_eq!(150, balance);
        } else {
            panic!("Could not get account from store");
        }
    }

    #[test]
    fn test_negative_balance() {
        let mut store = MemActStore::new();
        let client_id = 1;
        store.withdraw(client_id, 1);

        if let Some(act) = store.get_account(client_id) {
            assert_eq!(1, act.id());
            assert_eq!(0, act.available());
        } else {
            panic!("Could not get account from store");
        }
    }

    #[test]
    fn test_hold() {
        let mut store = MemActStore::new();
        let client_id = 1;
        store.deposit(client_id, 100);

        let avail = store.hold(client_id, 10);
        assert_eq!(90, avail);

        if let Some(act) = store.get_account(client_id) {
            assert_eq!(1, act.id());
            assert_eq!(90, act.available());
        } else {
            panic!("Could not get account from store");
        }

        assert_eq!(0, store.hold(client_id, 100));
        assert_eq!(10, store.unhold(client_id, 20));
    }

    #[test]
    fn test_lock() {
        let mut store = MemActStore::new();
        let client_id = 1;
        store.deposit(client_id, 100);
        let locked = store.lock_account(client_id);
        assert!(locked);

        if let Some(act) = store.get_account(client_id) {
            assert_eq!(1, act.id());
            assert!(act.is_locked());
        } else {
            panic!("Could not get account from store");
        }
    }
}
