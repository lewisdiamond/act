# Act

Parses the input transaction CSV and generates a CSV output with the final state
of each account.

## Assumptions

1. Available amounts in range i64::MAX and i64::MIN is sufficient for this user
   case.
2. Only deposits can be disputed.
3. Available amounts can be negative.
4. Transactions using more than 4 decimal places are considered errors and
   ignored (logged as warnings).
5. Chargebacks triggering withdrawals are allowed to result in negative total
   amounts as the account gets locked.

## Manual test file

`cargo run -- test-cases.csv`
Adding `-d/--debug` will print warnings for invalid transactions.
