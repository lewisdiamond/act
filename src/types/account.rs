use anyhow::{Result, anyhow, bail};
use serde::{Serialize, Serializer};
const PRECISION: u32 = 4;

#[derive(Debug, PartialEq)]
pub struct Account {
    id: u16,
    /// Total balance in the account.
    /// Can be negative if the account is in debt.
    total: i64,
    held: u64,
    locked: bool,
}

impl Account {
    pub fn new(client_id: u16) -> Account {
        Account {
            id: client_id,
            total: 0,
            held: 0,
            locked: false,
        }
    }

    pub fn with_balance(client_id: u16, seed_balance: i64) -> Account {
        Account {
            id: client_id,
            total: seed_balance,
            held: 0,
            locked: false,
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn available(&self) -> Result<i64> {
        self.total.checked_sub_unsigned(self.held).ok_or(anyhow!(
            "Overflow on available balance. Total: {}, Held: {}",
            self.total,
            self.held
        ))
    }
    pub fn held(&self) -> u64 {
        self.held
    }
    pub fn hold(&mut self, amnt: u64) -> Result<i64> {
        let current_held = self.held;
        if let Some(held) = self.held.checked_add(amnt) {
            self.held = held;
            self.available().inspect_err(|_| {
                self.held = current_held; // Rollback if available fails
            })
        } else {
            Err(anyhow!(
                "Overflow on hold. Current held: {}, Amount to hold: {}",
                self.held,
                amnt
            ))
        }
    }

    pub fn total(&self) -> i64 {
        self.total
    }

    pub fn unhold(&mut self, amnt: u64) -> Result<i64> {
        match self.held.checked_sub(amnt) {
            Some(held) => {
                self.held = held;
                self.available()
            }
            None => bail!(
                "Invalid unhold. Current held: {}, Amount to unhold: {}",
                self.held,
                amnt
            ),
        }
    }

    pub fn deposit(&mut self, amnt: u64) -> Result<i64> {
        match self.total.checked_add_unsigned(amnt) {
            Some(new_bal) => {
                self.total = new_bal;
                self.available()
            }
            None => Err(anyhow!(
                "Overflow on deposit. Current total: {}, Amount to deposit: {}",
                self.total,
                amnt
            )),
        }
    }
    fn internal_withdraw(&mut self, amnt: u64, allow_negative: bool) -> Result<i64> {
        let available = self.available()?;
        match available.checked_sub_unsigned(amnt) {
            Some(total) if total >= 0 || allow_negative => self.total = total,
            Some(total) => bail!("Withdrawal would result in negative balance: {}", total),
            None => bail!(
                "Withdrawal overflow. available={} withdraw={}",
                available,
                amnt
            ),
        }
        self.available()
    }
    pub fn withdraw(&mut self, amnt: u64) -> Result<i64> {
        self.internal_withdraw(amnt, false)
    }

    pub fn withdraw_allow_negative(&mut self, amnt: u64) -> Result<i64> {
        self.internal_withdraw(amnt, true)
    }

    pub fn lock(&mut self) -> bool {
        self.locked = true;
        self.locked
    }

    pub fn unlock(&mut self) -> bool {
        self.locked = false;
        self.locked
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct AccountSer {
    id: u16,
    total: String,
    held: String,
    available: String,
    locked: bool,
}

impl serde::Serialize for Account {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let dec = 10i64.pow(PRECISION);
        let udec = 10u64.pow(PRECISION);

        let total = format!("{}.{}", self.total / dec, self.total.abs() % dec);
        let held = format!("{}.{}", self.held / udec, self.held % udec);
        let available = self.available().map_or(String::from("0"), |a| {
            format!("{}.{}", a / dec, a.abs() % dec)
        });
        let ser = AccountSer {
            id: self.id,
            total,
            held,
            available,
            locked: self.locked,
        };
        ser.serialize(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new(100);
        assert_eq!(account.id(), 100);
        assert_eq!(account.total(), 0);
        assert_eq!(account.held(), 0);
        assert!(!account.is_locked());
    }

    #[test]
    fn test_account_with_balance() {
        let account = Account::with_balance(200, 500);
        assert_eq!(account.id(), 200);
        assert_eq!(account.total(), 500);
        assert_eq!(account.held(), 0);
    }
    #[test]
    fn test_account_deposit() {
        let mut account = Account::new(300);
        let balance = account.deposit(200);
        assert!(matches!(balance, Ok(200)));
        assert_eq!(account.total(), 200);
        assert!(matches!(account.available(), Ok(200)));
    }

    #[test]
    fn test_account_withdraw() {
        let mut account = Account::with_balance(400, 300);
        let balance = account.withdraw(100);
        assert!(matches!(balance, Ok(200)));
        assert_eq!(account.total(), 200);
        assert!(matches!(account.available(), Ok(200)));
    }

    #[test]
    fn test_account_hold() {
        let mut account = Account::with_balance(500, 300);
        let balance = account.hold(100);
        assert!(matches!(balance, Ok(200)));
        assert_eq!(account.held(), 100);
        assert!(matches!(account.available(), Ok(200)));
    }

    #[test]
    fn test_account_hold_overflow() {
        let mut account = Account::with_balance(1, i64::MAX);
        assert!(account.hold(u64::MAX).is_ok());
        account
            .hold(1)
            .expect_err("Hold should fail due to overflow");
    }
    #[test]
    fn test_account_unhold() {
        let mut account = Account::with_balance(600, 300);
        account.hold(100).expect("Hold failed");
        let balance = account.unhold(50);
        assert!(matches!(balance, Ok(250)));
        assert_eq!(account.held(), 50);
        assert!(matches!(account.available(), Ok(250)));
    }

    #[test]
    fn test_account_unhold_over() {
        let mut account = Account::with_balance(700, 300);
        account.hold(100).expect("Hold failed");
        account
            .unhold(200)
            .expect_err("Unhold should fail due to insufficient held amount");
    }
    #[test]
    fn test_withdraw_negative_balance() {
        let mut account = Account::new(800);
        account
            .withdraw(100)
            .expect_err("Withdraw should fail due to insufficient balance");
        assert_eq!(account.total(), 0);
        assert_eq!(account.held(), 0);
    }

    #[test]
    fn test_limit_values() {
        let mut account = Account::new(999);
        let balance = account
            .deposit(i64::MAX as u64)
            .expect("Max deposit should succeed");
        assert_eq!(balance, i64::MAX);
        account
            .deposit(1)
            .expect_err("Above max deposit should fail");
        assert_eq!(
            account.available().expect("Balance not available"),
            i64::MAX
        );
        let balance = account.hold(u64::MAX).expect("Holding max should succeed");
        assert_eq!(balance, i64::MIN);
        account.hold(1).expect_err("Holding beyong max should fail");
        assert_eq!(
            account.available().expect("Balance not available"),
            i64::MIN
        );
        let balance = account
            .unhold(u64::MAX)
            .expect("Holding to zero should succeed");
        assert_eq!(balance, i64::MAX);
        account.withdraw(1).expect("Withdraw 1 should succeed");
        account
            .hold(u64::MAX)
            .expect_err("Holding beyong max should fail");
        assert_eq!(
            account.available().expect("Balance not available"),
            i64::MAX - 1
        );
    }
}
