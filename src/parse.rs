use crate::types::Transaction;
use async_stream::stream;
use std::io::Read;
use tokio_stream::Stream;

pub fn parse<R: Read>(input: R) -> impl Stream<Item = Transaction> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .has_headers(true)
        .from_reader(input);

    stream! {
        for tx in reader.deserialize() {
            match tx {
                Ok(tx) => yield tx,
                Err(e) => eprintln!("Error reading CSV: {}", e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Transaction;
    use crate::types::TransactionType;
    use tokio_stream::StreamExt;
    use tokio_test::block_on;

    #[test]
    fn valid_csv_is_parsed() {
        block_on(async {
            let data = "\
type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
withdrawal,2,5,3.0";

            let expected = vec![
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
            let txs = parse(data.as_bytes()).collect::<Vec<Transaction>>().await;
            assert_eq!(expected, txs);
        });
    }

    #[test]
    fn valid_csv_with_whitespaces_is_parsed() {
        block_on(async {
            let data = "\
type,       client,  tx, amount
deposit,         1,   1,    1.0
deposit,         2,   2,    2.0
deposit,         1,   3,    2.0
withdrawal,      1,   4,    1.5
withdrawal,      2,   5,    3.0";

            let expected = vec![
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
            let txs = parse(data.as_bytes()).collect::<Vec<Transaction>>().await;
            assert_eq!(expected, txs);
        });
    }

    #[test]
    fn amounts_are_parsed_correctly() {
        block_on(async {
            let data = "\
type,client,tx,amount
deposit,1,1,1.0001
deposit,2,2,2.0010
deposit,1,3,10.01
withdrawal,1,4,01.10
withdrawal,2,5,10.0110";

            let expected = vec![
                Transaction {
                    tx_type: TransactionType::Deposit,
                    amount: 10001,
                    client: 1,
                    tx: 1,
                },
                Transaction {
                    tx_type: TransactionType::Deposit,
                    amount: 20010,
                    client: 2,
                    tx: 2,
                },
                Transaction {
                    tx_type: TransactionType::Deposit,
                    amount: 100100,
                    client: 1,
                    tx: 3,
                },
                Transaction {
                    tx_type: TransactionType::Withdrawal,
                    amount: 11000,
                    client: 1,
                    tx: 4,
                },
                Transaction {
                    tx_type: TransactionType::Withdrawal,
                    amount: 100110,
                    client: 2,
                    tx: 5,
                },
            ];
            let txs = parse(data.as_bytes()).collect::<Vec<Transaction>>().await;
            assert_eq!(expected, txs);
        });
    }

    #[test]
    fn invalid_amounts_are_filtered() {
        block_on(async {
            let data = "\
type,client,tx,amount
deposit,1,1,99999999999999999
deposit,2,2,18446744073709551615
deposit,1,3,18446744073709551616
withdrawal,1,4,0
withdrawal,1,4,
withdrawal,1,4,a
withdrawal,2,5,-1
withdrawal,1,6,-99999999999999999
withdrawal,1,6,-18446744073709551615
withdrawal,1,7,-18446744073709551616
withdrawal,1,7,1.010101";

            let expected = vec![
                Transaction {
                    tx_type: TransactionType::Withdrawal,
                    amount: 0,
                    client: 1,
                    tx: 4,
                },
                Transaction {
                    tx_type: TransactionType::Withdrawal,
                    amount: 0,
                    client: 1,
                    tx: 4,
                },
            ];
            let txs = parse(data.as_bytes()).collect::<Vec<Transaction>>().await;

            assert_eq!(expected, txs);
        });
    }

    #[test]
    fn disputes_are_parsed_correctly() {
        block_on(async {
            let data = "\
type,client,tx,amount
dispute,1,1,
dispute,1,2,";

            let expected = vec![
                Transaction {
                    tx_type: TransactionType::Dispute,
                    amount: 0,
                    client: 1,
                    tx: 1,
                },
                Transaction {
                    tx_type: TransactionType::Dispute,
                    amount: 0,
                    client: 1,
                    tx: 2,
                },
            ];
            let txs = parse(data.as_bytes()).collect::<Vec<Transaction>>().await;

            assert_eq!(expected, txs);
        });
    }
}
