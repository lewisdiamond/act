use crate::{
    stores::ActStore,
    types::{Transaction, TransactionType},
};
use std::collections::HashMap;

pub fn process(
    t: Transaction,
    act_store: &mut dyn ActStore,
    tx_store: &mut HashMap<u32, Transaction>,
) {
    match t.tx_type {
        TransactionType::Deposit => {
            act_store.deposit(t.client, t.amount);
            tx_store.insert(t.tx, t);
        }
        TransactionType::Withdrawal => {
            act_store.withdraw(t.client, t.amount);
        }
        TransactionType::Dispute => {
            if let Some(orig) = tx_store.get(&t.tx) {
                act_store.hold(t.client, orig.amount);
            }
        }
        TransactionType::Resolve => {
            if let Some(orig) = tx_store.get(&t.tx) {
                act_store.unhold(t.client, orig.amount);
            }
        }
        TransactionType::Chargeback => {
            if let Some(orig) = tx_store.get(&t.tx) {
                act_store.unhold(t.client, orig.amount);
                act_store.withdraw(t.client, orig.amount);
                act_store.lock_account(t.client);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores::MemActStore;
    use crate::types::Transaction;
    use crate::types::TransactionType;

    #[test]
    fn valid_tx_and_over_limit_withdraw() {
        let mut act_store: Box<dyn ActStore> = Box::new(MemActStore::new());
        let mut tx_store: HashMap<u32, Transaction> = HashMap::new();
        let txs = vec![
            Transaction {
                tx_type: TransactionType::Deposit,
                amount: 10000,
                client: 1,
                tx: 1,
            },
            Transaction {
                tx_type: TransactionType::Deposit,
                amount: 20000,
                client: 2,
                tx: 2,
            },
            Transaction {
                tx_type: TransactionType::Deposit,
                amount: 20000,
                client: 1,
                tx: 3,
            },
            Transaction {
                tx_type: TransactionType::Withdrawal,
                amount: 15000,
                client: 1,
                tx: 4,
            },
            Transaction {
                tx_type: TransactionType::Withdrawal,
                amount: 30000,
                client: 2,
                tx: 5,
            },
        ];
        for tx in txs {
            process(tx, act_store.as_mut(), &mut tx_store);
        }

        let act = act_store.get_account(1).unwrap();
        assert_eq!(0, act.held());
        assert_eq!(15000, act.available());

        let act = act_store.get_account(2).unwrap();
        assert_eq!(0, act.held());
        assert_eq!(20000, act.available());
    }

    #[test]
    fn held_funds() {
        let mut act_store: Box<dyn ActStore> = Box::new(MemActStore::new());
        let mut tx_store: HashMap<u32, Transaction> = HashMap::new();
        process(
            Transaction {
                tx_type: TransactionType::Deposit,
                amount: 10000,
                client: 1,
                tx: 1,
            },
            act_store.as_mut(),
            &mut tx_store,
        );
        process(
            Transaction {
                tx_type: TransactionType::Dispute,
                amount: 0,
                client: 1,
                tx: 1,
            },
            act_store.as_mut(),
            &mut tx_store,
        );
        let act = act_store.get_account(1).unwrap();
        assert_eq!(10000, act.held());
        assert_eq!(0, act.available());
        process(
            Transaction {
                tx_type: TransactionType::Withdrawal,
                amount: 10000,
                client: 1,
                tx: 2,
            },
            act_store.as_mut(),
            &mut tx_store,
        );
        let act = act_store.get_account(1).unwrap();
        assert_eq!(10000, act.held());
        assert_eq!(0, act.available());
        process(
            Transaction {
                tx_type: TransactionType::Resolve,
                amount: 0,
                client: 1,
                tx: 1,
            },
            act_store.as_mut(),
            &mut tx_store,
        );
        let act = act_store.get_account(1).unwrap();
        assert_eq!(0, act.held());
        assert_eq!(10000, act.available());
    }
}
