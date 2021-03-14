use serde::{Serialize, Serializer};
const PRECISION: u32 = 4;

#[derive(Debug, PartialEq)]
pub struct Account {
    id: u16,
    total: u64,
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

    pub fn with_balance(client_id: u16, seed_balance: u64) -> Account {
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
    pub fn available(&self) -> u64 {
        self.total.saturating_sub(self.held)
    }
    pub fn held(&self) -> u64 {
        self.held
    }
    pub fn hold(&mut self, amnt: u64) -> u64 {
        if let Some(new_held) = self.held.checked_add(amnt) {
            self.held = new_held;
        }
        self.available()
    }

    pub fn unhold(&mut self, amnt: u64) -> u64 {
        if let Some(new_held) = self.held.checked_sub(amnt) {
            self.held = new_held;
        }
        self.available()
    }
    pub fn deposit(&mut self, amnt: u64) -> u64 {
        if let Some(new_bal) = self.total.checked_add(amnt) {
            self.total = new_bal;
        };
        self.available()
    }
    pub fn withdraw(&mut self, amnt: u64) -> u64 {
        if self.available().checked_sub(amnt).is_some() {
            self.total -= amnt;
        };
        self.available()
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
        let total = format!(
            "{0:.4}",
            (self.total as f64) / (10u64.pow(PRECISION) as f64)
        );
        let held = format!("{0:.4}", (self.held as f64) / (10u64.pow(PRECISION) as f64));
        let available = format!(
            "{0:.4}",
            (self.available() as f64) / (10u64.pow(PRECISION) as f64)
        );
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
