use anyhow::{Result, anyhow};
use serde::de;
use serde::{Deserialize, Deserializer};
const PRECISION: u32 = 4;

#[derive(Eq, PartialEq, Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    pub client: u16,
    pub tx: u32,
    /// Amount of the smallest unit, e.g. 0.0001 as per the specification
    #[serde(deserialize_with = "de_amount")]
    pub amount: u64,
}

fn de_amount<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let deserialized = String::deserialize(deserializer)?;
    let mut split = deserialized.split('.');
    let units = split.next().map_or("0", |v| match v {
        "" => "0",
        _ => v,
    });

    let dec = match split.next() {
        Some(v) if v.len() > PRECISION as usize => {
            Err(anyhow!("decimal precision not supported: {}", v))
        }
        Some(v) => Ok(v),
        None => Ok("0"),
    }
    .map(|v| format!("{:0<4.4}", v))
    .map_err(de::Error::custom)?;
    (units.to_string() + &dec)
        .parse::<u64>()
        .map_err(de::Error::custom)
}
