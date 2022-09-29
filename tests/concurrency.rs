// tests processing multiple transaction files
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rust_decimal_macros::dec;

use tx_engine::common::*;

fn do_process_csv(accounts : &mut HashMap<u16, Account>, csv : &str)
{
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(accounts);
    engine.process_transactions(&mut data,&mut None);
}

fn do_process_path(accounts : &mut HashMap<u16, Account>, path : &str)
{
    let mut data = csv::Reader::from_path(path).unwrap();
    let mut engine = tx_engine::engine::Engine::new(accounts);
    engine.process_transactions(&mut data,&mut None);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn process_double()
{
    let csv = 
    "type,       client,     tx,     amount
    deposit,    1,           1,        1.0
    deposit,    2,           2,        2.0
    deposit,    1,           3,        2.0
    withdrawal, 1,           4,        1.5
    withdrawal, 2,           5,        3.0
    dispute,    2,           5,";

    let accounts : HashMap<u16, Account> = HashMap::new();
    let accounts = Arc::new(Mutex::new(accounts));

    let mut handles = vec![];

    for _ in 0..2
    {   
        let accounts = Arc::clone(&accounts);
        let handle = tokio::spawn( async move {

        let shared_accounts = &mut *(accounts.lock().unwrap());
        do_process_csv(shared_accounts, csv);

        });
        handles.push(handle);
    }

    for handle in handles
    {
        handle.await.unwrap();
    }

    let shared_accounts = &*(accounts.lock().unwrap());

    let account : &Account = 
    shared_accounts.get(&1).unwrap();

    // the end result should be such that both accounts have 
    // double the result, since we ran two processes of the same
    // transactions file concurrently.
    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(3.0));
    assert_eq!(account.locked, false);
}


#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn process_multiple_large_transactions()
{
    // will simulate many large transactions coming in at different times
    // to be processed concurrently.


    // this is a large file whose final balance on
    // client 2 is 4.5 available and  0.0 held
    // it essentially processes many useless transactions
    let path = "tests/large_transactions.csv";

    let accounts : HashMap<u16, Account> = HashMap::new();
    let accounts = Arc::new(Mutex::new(accounts));
    let num_spawns = 5000;

    let mut handles = vec![];

    // process thousands
    for i in 0..num_spawns
    {   
        let accounts = Arc::clone(&accounts);
        let accounts = Arc::clone(&accounts);

        // spawn threads here
        let handle = tokio::spawn(  async move  {

        let shared_accounts = &mut *(accounts.lock().unwrap());
        do_process_path(shared_accounts, path);

        // this wont be captured
        println!("process {i} complete.");

        });
        handles.push(handle);

    }

    for handle in handles
    {
        handle.await.unwrap();
    }

    let shared_accounts = &*(accounts.lock().unwrap());

    let account : &Account = 
    shared_accounts.get(&2).unwrap();
    assert_eq!(account.held, dec!(0.0));

    //  4.5 * num_spawns
    assert_eq!(account.available, dec!(22500.0));
    assert_eq!(account.locked, false);

}