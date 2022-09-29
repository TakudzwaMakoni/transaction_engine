use tx_engine::common::*;
use std::collections::HashMap;
use rust_decimal_macros::dec;

#[test]
fn process_deposit()
{

    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      1,        5.0";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    engine.process_transactions(&mut data,&mut None);

    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&1).unwrap();

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(5.0));
    assert_eq!(account.locked, false);
    assert_eq!(tx.disputed, false);

}

#[test]
fn process_withdrawal()
{

    let csv=
    "type,       client,     tx,     amount
    deposit,         1,      1,        5.0
    withdrawal,      1,      2,        4.0";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    engine.process_transactions(&mut data,&mut None);

    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&1).unwrap();

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(1.0));
    assert_eq!(account.locked, false);
    assert_eq!(tx.disputed, false);
}

#[test]
fn process_withdrawal_insufficient_amount()
{

    let csv=
    "type,       client,     tx,     amount
    deposit,         1,      1,        5.0
    withdrawal,      1,      2,        6.0";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    let mut logger = Logger::new(&"tests/testlog.txt".to_string());
    engine.process_transactions(&mut data,&mut logger);

    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&1).unwrap();

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(5.0));
    assert_eq!(account.locked, false);
    assert_eq!(tx.disputed, false);

    let last_event = logger.unwrap().last_entry();
    assert_eq!(last_event, ProcessEvent::ErrInsufficientfunds(1,2));
}

#[test]
fn process_dispute_deposit()
{

    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      3,     5.0
    deposit,         2,      4,     7.0
    dispute,         1,      3,";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    engine.process_transactions(&mut data,&mut None);

    // dispute for client 1 should be successful
    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&3).unwrap();
    assert_eq!(tx.disputed, true);

    assert_eq!(account.held, dec!(5.0));
    assert_eq!(account.available, dec!(0.0));
    assert_eq!(account.locked, false);
            
}

#[test]
fn process_invalid_tx_dispute_deposit()
{

    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      3,     5.0
    deposit,         2,      4,     7.0
    dispute,         1,      3,
    dispute,         2,      5,"; 

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    let mut logger = Logger::new(&"tests/testlog.txt".to_string());
    engine.process_transactions(&mut data,&mut logger);

    // dispute for client 2 is unsuccessful because
    // it references a non existent tx id (5).
    // shouldn't panic: account remains unchanged.

    let account : &Account = 
    engine.accounts.get(&2).unwrap();

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(7.0));
    assert_eq!(account.locked, false);

    // check log to assert non existent tx id event ackknowledged
    let last_event = logger.unwrap().last_entry();
    assert_eq!(last_event, ProcessEvent::ErrTxNotFound(5));

}

#[test]
fn process_dispute_withdrawal()
{

    let csv = 
    "type,       client,     tx,     amount
    deposit,         1,      3,     5.0
    withdrawal,      1,      4,     4.0      
    dispute,         1,      4,
    deposit,         2,      5,     7.0
    withdrawal,      2,      6,     6.0      
    dispute,         2,      7,";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    engine.process_transactions(&mut data,&mut None);

    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&4).unwrap();
    assert_eq!(tx.disputed, true);

    assert_eq!(account.held, dec!(4.0));
    assert_eq!(account.available, dec!(-3.0));
    assert_eq!(account.locked, false);
}

#[test]
fn process_invalid_tx_dispute_withdrawal()
{

        let csv = 
        "type,       client,     tx,     amount
        deposit,         1,      3,     5.0
        withdrawal,      1,      4,     4.0      
        dispute,         1,      4,
        deposit,         2,      5,     7.0
        withdrawal,      2,      6,     6.0      
        dispute,         2,      7,";

        let mut accounts : HashMap<u16, Account> = HashMap::new();
        let mut data = csv::Reader::from_reader(csv.as_bytes());
        let mut engine = tx_engine::engine::Engine::new(&mut accounts);
        let mut logger = Logger::new(&"tests/testlog.txt".to_string());
        engine.process_transactions(&mut data,&mut logger);

        // dispute for client 2 is unsuccessful because
        // it references a non existent tx id (7).
        // shouldn't panic: account remains unchanged.
    
        let account : &Account = 
        engine.accounts.get(&2).unwrap();

        assert_eq!(account.held, dec!(0.0));
        assert_eq!(account.available, dec!(1.0));
        assert_eq!(account.locked, false);

        // check log to assert non existent tx id event ackknowledged
        let last_event = logger.unwrap().last_entry();
        assert_eq!(last_event, ProcessEvent::ErrTxNotFound(7));
}

#[test]
fn process_resolve_deposit()
{
    let csv = 
    "type,       client,     tx,     amount
    deposit,         1,      5,     5.0
    dispute,         1,      5,
    resolve,         1,      5,";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    engine.process_transactions(&mut data,&mut None);

    // resolve dispute for client 1 should be successful
    
    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&5).unwrap();
    assert_eq!(tx.disputed, false);

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(5.0));
    assert_eq!(account.locked, false);

}

#[test]
fn process_resolve_withdrawal()
{

    let csv = 
    "type,       client,     tx,     amount
    deposit,         1,      2,     5.0
    withdrawal,      1,      3,     2.0
    dispute,         1,      3,
    resolve,         1,      3,";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    engine.process_transactions(&mut data,&mut None);

    // resolve dispute for client 1 should be successful
    
    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&3).unwrap();
    assert_eq!(tx.disputed, false);

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(3.0));
    assert_eq!(account.locked, false);

}

#[test]
fn process_invalid_tx_resolve_deposit()
{
    
    let csv = 
    "type,       client,     tx,     amount
    deposit,         1,      5,     5.0
    dispute,         1,      5,
    resolve,         1,      5,
    deposit,         2,      6,     7.0
    dispute,         2,      6,
    resolve,         2,      7,";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    let mut logger = Logger::new(&"tests/testlog.txt".to_string());
    engine.process_transactions(&mut data,&mut logger);

    // resolve for client 2 is unsuccessful because
    // it references tx id (7), which doesn't exist yet - 
    // since we can assume transactions are in chronological
    // or respectful to that of the file.
    // shouldn't panic: account remains unchanged.
    
    let account : &Account = 
    engine.accounts.get(&2).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&6).unwrap();
    assert_eq!(tx.disputed, true);

    assert_eq!(account.held, dec!(7.0));
    assert_eq!(account.available, dec!(0.0));
    assert_eq!(account.locked, false);

    let last_event = logger.unwrap().last_entry();
    assert_eq!(last_event, ProcessEvent::ErrTxNotFound(7));
}

#[test]
fn process_undisputed_tx_resolve_deposit()
{
    
    let csv = 
    "type,       client,     tx,     amount
    deposit,         1,      5,     5.0
    resolve,         1,      5,";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    let mut logger = Logger::new(&"tests/testlog.txt".to_string());
    engine.process_transactions(&mut data,&mut logger);

    // resolve for client 2 is unsuccessful because
    // it references tx id (7), which is undisputed.
    // shouldn't panic: account remains unchanged.
    
    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&5).unwrap();
    assert_eq!(tx.disputed, false);

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(5.0));
    assert_eq!(account.locked, false);

    let last_event = logger.unwrap().last_entry();
    assert_eq!(last_event, ProcessEvent::ErrTxNotDisputed(5));
}


#[test]
fn process_chargeback_deposit()
{
    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      3,     5.0
    dispute,         1,      3,
    chargeback,      1,      3,";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    engine.process_transactions(&mut data,&mut None);

    // chargeback for client 1 should be successful
    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&3).unwrap();
    assert_eq!(tx.disputed, false);

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(0.0));
    assert_eq!(account.locked, true);
}

#[test]
fn process_chargeback_withdrawal()
{
    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      3,     5.0
    withdrawal,      1,      4,     3.0
    dispute,         1,      4,
    chargeback,      1,      4,";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    engine.process_transactions(&mut data,&mut None);

    // chargeback for client 1 should be successful
    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&4).unwrap();
    assert_eq!(tx.disputed, false);

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(-1.0));
    assert_eq!(account.locked, true);
}

#[test]
fn process_invalid_tx_chargeback_withdrawal()
{
    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      3,     5.0
    withdrawal,      1,      4,     3.0
    dispute,         1,      4,
    chargeback,      1,      7,";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    let mut logger = Logger::new(&"tests/testlog.txt".to_string());
    engine.process_transactions(&mut data,&mut logger);

    // chargeback for client 1 should be successful
    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&4).unwrap();
    assert_eq!(tx.disputed, true);

    assert_eq!(account.held, dec!(3.0));
    assert_eq!(account.available, dec!(-1.0));
    assert_eq!(account.locked, false);

    let last_event = logger.unwrap().last_entry();
    assert_eq!(last_event, ProcessEvent::ErrTxNotFound(7));
}

#[test]
fn process_undisputed_chargeback_deposit()
{
    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      3,     5.0
    withdrawal,      1,      4,     3.0
    chargeback,      1,      3,";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    let mut logger = Logger::new(&"tests/testlog.txt".to_string());
    engine.process_transactions(&mut data,&mut logger);

    // chargeback for client 1 should fail
    // because the referenced tx is undisputed
    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&3).unwrap();
    assert_eq!(tx.disputed, false);

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(2.0));
    assert_eq!(account.locked, false);

    let last_event = logger.unwrap().last_entry();
    assert_eq!(last_event, ProcessEvent::ErrTxNotDisputed(3));
}

#[test]
fn process_undisputed_chargeback_withdrawal()
{
    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      3,     5.0
    withdrawal,      1,      4,     3.0
    chargeback,      1,      4,";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    let mut logger = Logger::new(&"tests/testlog.txt".to_string());
    engine.process_transactions(&mut data,&mut logger);

    // chargeback for client 1 should fail
    // because the referenced tx is undisputed
    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&4).unwrap();
    assert_eq!(tx.disputed, false);

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(2.0));
    assert_eq!(account.locked, false);

    let last_event = logger.unwrap().last_entry();
    assert_eq!(last_event, ProcessEvent::ErrTxNotDisputed(4));
}


#[test]
fn process_invalid_auth_chargeback()
{
    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      3,     5.0
    deposit,         2,      4,     3.0
    chargeback,      2,      3,";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    let mut logger = Logger::new(&"tests/testlog.txt".to_string());
    engine.process_transactions(&mut data,&mut logger);

    // chargeback for client 2 is unsuccessful because
    // it references tx id (3), which it doesn't own.
    // shouldn't panic: account remains unchanged.

    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    let tx : &Tx = 
    engine.tx_history.get(&3).unwrap();
    assert_eq!(tx.disputed, false);

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(5.0));
    assert_eq!(account.locked, false);

    let last_event = logger.unwrap().last_entry();
    assert_eq!(last_event, ProcessEvent::ErrUnauthorisedTx(2,3));
}

#[test]
fn process_negative_deposit()
{

    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      3,     -5.0";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    let mut logger = Logger::new(&"tests/testlog.txt".to_string());
    engine.process_transactions(&mut data,&mut logger);

    // deposit for client 1 should be fail
    // because the amount is negative.
    //( therefore tx wont exist.)
    if let Some(_) = engine.tx_history.get(&3)
    {
        panic!()
    }
 
    let last_event = logger.unwrap().last_entry();
    assert_eq!(last_event, ProcessEvent::ErrAmountNegative(3));


}

#[test]
fn process_negative_withdrawal()
{

    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      3,     5.0
    withdrawal,      1,      4,     -5.0";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    let mut logger = Logger::new(&"tests/testlog.txt".to_string());
    engine.process_transactions(&mut data,&mut logger);

    // withdrawal for client 1 should be fail
    // because the amount is negative
    let account : &Account = 
    engine.accounts.get(&1).unwrap();


    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(5.0));
    assert_eq!(account.locked, false);

    let last_event = logger.unwrap().last_entry();
    assert_eq!(last_event, ProcessEvent::ErrAmountNegative(4));   
}


#[test]
fn process_invalid_type_()
{

    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      3,     5.0
    withdfrawal,      1,      4,     5.0";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    let mut logger = Logger::new(&"tests/testlog.txt".to_string());
    engine.process_transactions(&mut data,&mut logger);

    // withdrawal for client 1 should be fail
    // because the the type has a typo.
    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(5.0));
    assert_eq!(account.locked, false);

    let last_event = logger.unwrap().last_entry();
    assert_eq!(last_event, ProcessEvent::ErrUnrecognisedTx(1,"withdfrawal"
                                                             .to_string()));   
}

#[test]
fn process_duplicate_tx_ids_()
{

    let csv =
    "type,       client,     tx,     amount
    deposit,         1,      3,     5.0
    withdrawal,      1,      3,     5.0";

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut data = csv::Reader::from_reader(csv.as_bytes());
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    let mut logger = Logger::new(&"tests/testlog.txt".to_string());
    engine.process_transactions(&mut data,&mut logger);

    // withdrawal for client 1 should be fail
    // because it uses a tx id which already exists.
    let account : &Account = 
    engine.accounts.get(&1).unwrap();

    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.available, dec!(5.0));
    assert_eq!(account.locked, false);

    let last_event = logger.unwrap().last_entry();
    assert_eq!(last_event, ProcessEvent::ErrTxIdExists(3));   
}