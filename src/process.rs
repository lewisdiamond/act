use crate::{
    stores::ActStore,
    types::{Transaction, TransactionType},
};
use anyhow::{Result, anyhow, bail};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct InternalTransaction {
    tx: Transaction,
    disputed: bool,
}

/// Processes a transaction by updating the account store and transaction store.
pub fn process(
    t: Transaction,
    act_store: &mut dyn ActStore,
    tx_store: &mut HashMap<u32, InternalTransaction>,
) -> Result<i64> {
    match t.tx_type {
        TransactionType::Deposit => act_store.deposit(t.client, t.amount).inspect(|_| {
            tx_store.insert(t.tx, InternalTransaction{ tx: t, disputed: false});
        }),
        TransactionType::Withdrawal => act_store.withdraw(t.client, t.amount),
        TransactionType::Dispute => tx_store
            .get_mut(&t.tx)
            .ok_or(anyhow!("Dispute: Transaction not found in store"))
            .and_then(|tx| {
                if t.client != tx.tx.client {
                    bail!(
                        "Dispute: client mismatch. Transaction client: {}, Dispute client: {}",
                        tx.tx.client,
                        t.client
                    )
                }
                if tx.disputed {
                    bail!("Dispute: Transaction already disputed")
                }
                tx.disputed = true;
                act_store.hold(t.client, tx.tx.amount)
            }),
        TransactionType::Resolve => tx_store
            .get_mut(&t.tx)
            .ok_or(anyhow!("Resolve: Transaction not found in store"))
            .and_then(|tx| {
                if t.client != tx.tx.client {
                    bail!(
                        "Resolve: client mismatch. Transaction client: {}, Resolve client: {}",
                        tx.tx.client,
                        t.client
                    )
                }
                if !tx.disputed {
                    bail!("Resolve: Transaction not disputed")
                }
                tx.disputed = false;
                act_store.unhold(t.client, tx.tx.amount)
            }),
        TransactionType::Chargeback => {
            tx_store
                .get(&t.tx)
                .ok_or(anyhow!("Chargeback: Transaction not found in store"))
                .and_then(|tx| {
                if t.client != tx.tx.client {
                        bail!(
                            "Chargeback: client mismatch. Transaction client: {}, Chargeback client: {}",
                            tx.tx.client,
                            t.client
                        )
                }
                if !tx.disputed {
                    bail!("Chargeback: Transaction not disputed")
                }
                act_store.lock_account(t.client);
                act_store.unhold(t.client, tx.tx.amount)?;
                act_store.withdraw_unchecked(t.client, tx.tx.amount)
            })
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
        let mut tx_store: HashMap<u32, InternalTransaction> = HashMap::new();
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
        ];
        for tx in txs {
            process(tx, act_store.as_mut(), &mut tx_store).unwrap();
        }
        process(
            Transaction {
                tx_type: TransactionType::Withdrawal,
                amount: 30000,
                client: 2,
                tx: 5,
            },
            act_store.as_mut(),
            &mut tx_store,
        )
        .expect_err("Withdrawal should fail due to insufficient funds");

        let act = act_store.get_account(1).unwrap();
        assert_eq!(0, act.held());
        assert!(matches!(act.available(), Ok(15000)));

        let act = act_store.get_account(2).unwrap();
        assert_eq!(0, act.held());
        assert!(matches!(act.available(), Ok(20000)));
    }

    #[test]
    fn held_funds() {
        let mut act_store: Box<dyn ActStore> = Box::new(MemActStore::new());
        let mut tx_store: HashMap<u32, InternalTransaction> = HashMap::new();
        process(
            Transaction {
                tx_type: TransactionType::Deposit,
                amount: 10000,
                client: 1,
                tx: 1,
            },
            act_store.as_mut(),
            &mut tx_store,
        )
        .unwrap();
        process(
            Transaction {
                tx_type: TransactionType::Dispute,
                amount: 0,
                client: 1,
                tx: 1,
            },
            act_store.as_mut(),
            &mut tx_store,
        )
        .unwrap();
        let act = act_store.get_account(1).unwrap();
        assert_eq!(10000, act.held());
        assert!(matches!(act.available(), Ok(0)));
        process(
            Transaction {
                tx_type: TransactionType::Withdrawal,
                amount: 10000,
                client: 1,
                tx: 2,
            },
            act_store.as_mut(),
            &mut tx_store,
        )
        .expect_err("Withdrawal should fail due to held funds");
        let act = act_store.get_account(1).unwrap();
        assert_eq!(10000, act.held());
        assert!(matches!(act.available(), Ok(0)));
        process(
            Transaction {
                tx_type: TransactionType::Resolve,
                amount: 0,
                client: 1,
                tx: 1,
            },
            act_store.as_mut(),
            &mut tx_store,
        )
        .unwrap();
        let act = act_store.get_account(1).unwrap();
        assert_eq!(0, act.held());
        assert!(matches!(act.available(), Ok(10000)));
    }
    #[test]
    fn dispute_client_mismatch() {
        let mut act_store: Box<dyn ActStore> = Box::new(MemActStore::new());
        let mut tx_store: HashMap<u32, InternalTransaction> = HashMap::new();
        process(
            Transaction {
                tx_type: TransactionType::Deposit,
                amount: 10000,
                client: 1,
                tx: 1,
            },
            act_store.as_mut(),
            &mut tx_store,
        )
        .unwrap();
        process(
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 2,
                tx: 1,
                amount: 0,
            },
            act_store.as_mut(),
            &mut tx_store,
        )
        .expect_err("Dispute should fail due to client mismatch");
        let act = act_store.get_account(1).unwrap();
        assert_eq!(0, act.held());
        assert!(matches!(act.available(), Ok(10000)));
    }
    #[test]
    fn chargeback() {
        let mut act_store: Box<dyn ActStore> = Box::new(MemActStore::new());
        let mut tx_store: HashMap<u32, InternalTransaction> = HashMap::new();
        let txs = vec![
            Transaction {
                tx_type: TransactionType::Deposit,
                amount: 20000,
                client: 1,
                tx: 1,
            },
            Transaction {
                tx_type: TransactionType::Withdrawal,
                amount: 10000,
                client: 1,
                tx: 2,
            },
            Transaction {
                tx_type: TransactionType::Dispute,
                amount: 0,
                client: 1,
                tx: 1,
            },
        ];
        txs.into_iter().for_each(|tx| {
            process(tx, act_store.as_mut(), &mut tx_store).unwrap();
        });
        let act = act_store.get_account(1).unwrap();
        assert_eq!(20000, act.held());
        assert_eq!(10000, act.total());
        assert!(matches!(act.available(), Ok(-10000)));
        assert!(!act.is_locked());
        process(
            Transaction {
                tx_type: TransactionType::Chargeback,
                amount: 0,
                client: 1,
                tx: 1,
            },
            act_store.as_mut(),
            &mut tx_store,
        )
        .unwrap();
        let act = act_store.get_account(1).unwrap();
        assert_eq!(0, act.held());
        assert!(matches!(act.available(), Ok(-10000)));
        assert!(act.is_locked());
    }
    #[test]
    fn resolve() {
        let mut act_store: Box<dyn ActStore> = Box::new(MemActStore::new());
        let mut tx_store: HashMap<u32, InternalTransaction> = HashMap::new();
        process(
            Transaction {
                tx_type: TransactionType::Deposit,
                amount: 10000,
                client: 1,
                tx: 1,
            },
            act_store.as_mut(),
            &mut tx_store,
        )
        .unwrap();
        process(
            Transaction {
                tx_type: TransactionType::Dispute,
                amount: 0,
                client: 1,
                tx: 1,
            },
            act_store.as_mut(),
            &mut tx_store,
        )
        .unwrap();
        let act = act_store.get_account(1).unwrap();
        assert_eq!(10000, act.held());
        assert!(matches!(act.available(), Ok(0)));
        process(
            Transaction {
                tx_type: TransactionType::Resolve,
                amount: 0,
                client: 1,
                tx: 1,
            },
            act_store.as_mut(),
            &mut tx_store,
        )
        .unwrap();
        let act = act_store.get_account(1).unwrap();
        assert_eq!(0, act.held());
        assert!(matches!(act.available(), Ok(10000)));
    }

    #[test]
    fn lock_unlock_account() {
        let mut act_store: Box<dyn ActStore> = Box::new(MemActStore::new());
        let mut tx_store: HashMap<u32, InternalTransaction> = HashMap::new();
        process(
            Transaction {
                tx_type: TransactionType::Deposit,
                amount: 10000,
                client: 1,
                tx: 1,
            },
            act_store.as_mut(),
            &mut tx_store,
        )
        .unwrap();
        let locked = act_store.lock_account(1);
        assert!(locked);
        let locked = act_store.unlock_account(1);
        assert!(!locked);
        let act = act_store.get_account(1).unwrap();
        assert!(!act.is_locked());
    }
}
