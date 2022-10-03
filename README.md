# Assumptions based on the text:

Based on the given text, we are told that we can make the following assumptions about the input csv file:

> You can assume the type is a string, the client column is a valid u16 client ID, the tx is a valid u32 transaction ID, and the amount is a decimal value with a precision of up to four places past the decimal.

- The type is a string.
- The client column will contains valid u16 values.
- The tx Column will contain valid u32 values.
- The amount will contain a valid float with precision up to 4 d.p.
- Though unordeded, client IDs will be unique per client (there will be no duplicate clients).
- Though unordeded, transaction IDs will be unique per transaction.

I take this to mean that Withdrawal and Deposits are only considered transactions and only they will occur uniquely,
since dispute/resolves/chargebacks need to reference existing transaction ids.

# dispute/resolve/chargeback logic

I was also unclear about disputes: on one hand the text states that a dispute is a reversal of a transaction. This would have me think that disputing a withdrawal transaction means reversing that withdrawal with an equivalent deposit. 

But then the text explicitly states that in a dispute,

 >"The client's available funds should decrease by the  amount disputed, and their held funds should increase by the amount disputed".

 I deferred to this more explicit instruction.
    
One assumption I have made not given by the text is that client 'a' should not be able to dispute/resolve/chargeback a transaction belonging to client 'b', so I have added that check also, since the globality of transaction ids would make that in fact possible (i had to add checks for this anyway as part of testing to make sure I dont make a test csv that is incorrect in this way).

# My decision to process transactions as they are streamed

We will process each transaction as we stream it from the file. This way we are not having to loop the the file for preprocessing (e.g parsing and validation), and then loop through the preprocessed transactions again the next time to apply the transactions to accounts. We can do this with confidence in the integrity of the input file, because of the assumptions given to us by the text: that the format and types in the file are correct.

This means however if a fatal error occurs in between streaming and processing, modified client accounts could be corrupted. The kind of error I mean is if the server crashes or hardware fails. The easy solution is to keep a backup of accounts before any processing begins as recovery. In my experience, servers holding sensitive client data do this at the end or beginning of the day. I wont implement this beginning/end of day backup for this toy engine.

# Concurrency

We need to process each transaction serially and in the same order as the file, otherwise the engine could erroneously believe a withdrawal is invalid because an earlier deposit has not been processed yet.

However, assuming related transactions are kept to a single input file, we should be able to run multiple instances of processing transaction files concurrently, sharing the accounts over each thread, since there would be no conflicting references across files, and the balance once all threads are complete would simply reflect the aggregation of the transactions on the shared accounts. This would behave like a regular current account which may have money coming from one place and going out from another simultaneously.

Some concurrency tests have been written, they are located in the 'tests' folder.
To run the concurrency tests: `cargo test --test concurrency` from the root directory.

I am using tokio for this test file.

This test may take a while as it processes many larger files. To show the varying orders of completion for each worker, run with the --nocapture flag:

`cargo test --nocapture`.

# Memory usage

It is inevitable that we must retain copies of processed transactions, because they may be later referenced by dispute/resolve/chargeback.

This adds space complexity O(N). My experience with servers in the fintech industry has taught me that modern servers have a lot of memory to handle a large data, some organisations even use a real time database (RTD) which is essentially storing data in RAM persistently and using it as the database for the benefit of faster execution (often done for advanced trading where time of execution is critical).

Reading from file is magnitutes slower than from memory, and lookup would be order O(N) time complexity since I need to search each record, because the transaction ids are not guaranteed to be ordered. So I opted with reading from memory, this way the app would read and execute faster, and I can use a hashmap to insert and lookup previous transactions, and this would be linear time complexity O(1).

Therefore I have opted to store the transactions in memory - we would only need to store the withdrawal / deposit transactions since only these can get referenced.

# Tests

1. ** regression **

The test cases that I have handled, and expected behaviour of the app are documented in the regression tests, located in the tests directory.

To run the regression tests: `cargo test --test regression` from the root directory.

 2. ** Integration **

The concurrency tests are integration tests where I have put together the types created to make rudimentary concurrent app, which is supposed to simulate the kind of workload a server might recieve from thousands of streams simltaneously.

They mainly exist to test / demonstrate that the app can be ran concurrently over multiple threads, and will produce correct results.

3. ** Unit tests **

In src/common.rs there are some unit tests to test operations on the 'Account' type for processing different transactions:

'deposit' and 'withdraw', 'withhold' funds for disputes, 'release held' funds for resolve, and 'charge' for chargebacks.

These unit tests specifically test that these methods return values that are correct to 4 decimal places as required by the text.

to run all tests: `cargo test`

# Logging

The logging feature was not part of the exercise and was mainly for my own debugging purposes. It is very rudimentary, and not intended for consideration. The normal running of the app `cargo run -- <csv path>` effectively disables logging, but I am using it in some tests to 
verify events which occur during runtime but dont throw or cause the process to stop.

to run with logging:

`cargo run -- <csv path> <logfile destination path>`

where the second argument is the file to be written to or created.


# Running the program


`cargo run -- transactions.csv`

or replace the csv file with some other path.


# Additional note on module organisation

I had pains with creating a binary crate with integration / regression testing. 
As far as I could tell from searching, the most workable workaround is to have a lib.rs and
use provide mods in main.rs, this makes it easier to also include external files
in the tests directory.

Why am I using 
`
extern crate tx_engine;

use std::process;
use std::env;
use std::collections::HashMap;

...

use tx_engine::common::Logger;
use tx_engine::common::Account;
use tx_engine::common::ProcessEvent;

...

let mut engine = tx_engine::Engine::new(&mut accounts);

`
 and not 

 `
mod engine;
mod app_process;
mod common;

...

use common::Logger;
use common::Account;
use common::ProcessEvent;

...

let mut engine = engine::Engine::new(&mut accounts);

 `
 ?

 because rust-analyser will fail to resolve that I am indeed using the functions
 `last_entry` and `read` in my tests directory, and will output those warnings 
 on build. since that will likely factor in, I don't want that misleading warning.