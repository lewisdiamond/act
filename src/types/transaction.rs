use serde::de;
use serde::{Deserialize, Deserializer};
const PRECISION: u32 = 4;

#[derive(Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, PartialEq)]
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
    //TODO use something like num_bigint instead
    let deserialized = String::deserialize(deserializer)?;
    let mut splitted = deserialized.split('.');
    let units = splitted
        .next()
        .map_or(Ok(0), |v| match v {
            "" => Ok(0),
            _ => v.parse::<u64>(),
        })
        .map_err(de::Error::custom)?
        .checked_mul(10u64.pow(PRECISION))
        .ok_or_else(|| de::Error::custom("Value too large"))?;
    //TODO format! here isn't great!
    let dec = splitted
        .next()
        .map_or(Ok(0u64), |v| format!("{:0<4.4}", v).parse::<u64>())
        .map_err(de::Error::custom)?;

    Ok(units + dec)
}
