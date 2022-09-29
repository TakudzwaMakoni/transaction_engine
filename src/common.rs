use std::fs::OpenOptions;
use std::io::Write;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

use chrono;

// errors which occur during processing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessEvent
{
    StartOfLogger,
    ProcessComplete,
    ExternalErr(String),
    ErrTxNotFound(u32),
    ErrUnrecognisedTx(usize, String),
    ErrInsufficientfunds(u16, u32),
    ErrTxNotDisputed(u32),
    ErrUnauthorisedTx(u16,u32)
}

impl ProcessEvent
{
    pub fn info(&self)
    -> (String, chrono::DateTime<chrono::Local> )
    {
        return match &*self
        {
            ProcessEvent::StartOfLogger =>
            {
                (String::from("Event logger created."),
                chrono::offset::Local::now())
            }
            ProcessEvent::ExternalErr(msg) =>
            {
                (msg.to_string(),
                chrono::offset::Local::now())
            }
            ProcessEvent::ProcessComplete =>
            {
                (String::from("Processed completed."),
                chrono::offset::Local::now())
            }
            ProcessEvent::ErrTxNotFound(tx_id) =>
            {
                (format!("ProcessError: Transaction with id '{tx_id}' is not found"),
                chrono::offset::Local::now())
            }
            ProcessEvent::ErrUnrecognisedTx(line, tx_type) =>
            {
                (format!(
                    "ProcessError: In csv, line {line}: 
                    '{tx_type}' is not a recognised transaction type."
                    ), chrono::offset::Local::now())
            }
            ProcessEvent::ErrInsufficientfunds(cli_id, tx_id) =>
            {
                (format!("ProcessError: Client with id '{cli_id}' has \
                \ninsufficient funds for transaction with id {tx_id}"),
                chrono::offset::Local::now())
            }
            ProcessEvent::ErrTxNotDisputed(tx_id) =>
            {
                (format!("ProcessError: The referenced transaction with \
                \nid '{tx_id}' isn't under dispute.'"),
                chrono::offset::Local::now())
            }
            ProcessEvent::ErrUnauthorisedTx(cli_id, tx_id) =>
            {
                (format!("ProcessError: Client with id '{cli_id}' cannot \
                reference transaction with id '{tx_id}' because \
                they do not own the transaction.'"),
                chrono::offset::Local::now())
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tx
{
    pub id       : u32,
    pub client   : u16,
    pub amount   : Decimal,
    pub disputed    : bool
}

impl Tx
{
    pub fn new(tx_id: u32, cli_id:u16, am: Decimal, disp: bool) 
    -> Self
    { 
        Tx
        {
            id       : tx_id,
            client   : cli_id,
            amount   : am,
            disputed : disp
        }
    }
}

#[derive(Debug, Clone)]
pub struct Account
{
    pub available:  Decimal,
    pub held:       Decimal,
    pub locked:     bool
}

impl Account
{
    pub fn new() -> Self
    {
        Account 
        {
            available:  dec!(0.0).round_dp(4),
            held:       dec!(0.0).round_dp(4),
            locked:     false
        }
    }

    // deposit to available balance
    pub fn deposit( &mut self, amount : &Decimal)
    {
        self.available += amount.round_dp(4);
    }

    // withdraw from available balance
    pub fn withdraw( &mut self, amount : &Decimal)
    {
        self.available -= amount.round_dp(4);
    }

    // move funds from available balance to held balance.
    pub fn withhold( &mut self, amount : &Decimal)
    {
        self.available -= amount.round_dp(4);
        self.held += amount.round_dp(4);
    }

    // release held funds into available
    pub fn release_held( &mut self, amount : &Decimal)
    {
        self.available += amount.round_dp(4);
        self.held -= amount.round_dp(4);
    }

    // applies a chargeback on held funds.
    pub fn charge( &mut self, amount : &Decimal)
    {
        self.held -= amount.round_dp(4);
    }

    pub fn lock(&mut self)
    {
        self.locked = true;
    }

}

pub struct Logger
{
    pub log_file : std::fs::File,
    pub last_event : ProcessEvent
}

impl Logger
{
    pub fn new(path : &String)
    -> Option<Self>
    {
        match Option::from(
            OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path))
        {
            Some(result) => 
            {
                // let this unrwap panic if it fails
                return 
                Some(
                    Logger
                    {
                        log_file    : result.unwrap(),
                        last_event  : ProcessEvent::StartOfLogger 
                    });
            },
            None => 
            {
                return None;
            }
        }
    }

    pub fn log(&mut self, event : &ProcessEvent)
    {
        // let this panic if it fails
        let info        = event.info();
        let message     = info.0;
        let timestamp   = info.1;

        writeln!(self.log_file, "EVENT LOG {timestamp}:\n{message}\n")
                 .unwrap();
        self.last_event = event.clone();
    }

    // returns the last message with timestamp
    pub fn last_entry(&self)
    -> ProcessEvent
    {
        self.last_event.clone()
    }
}


// unit tests ////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod accounts_operations
{
    use super::*;

    #[test]
    fn test_deposit_precision_to_account()
    {
        // expect back 4 d.p precision
        let mut account = Account::new();
        //assert_eq!(account.available, &dec!(0.0));
        account.deposit(&dec!(12.3456789));
        assert_eq!(account.available, dec!(12.3457));

    }

    #[test]
    fn test_withdrawal_precision_to_account()
    {
        // expect back 4 d.p precision
        let mut account = Account::new();
        //assert_eq!(account.available, &dec!(0.0));
        account.deposit(&dec!(12.3456789));
        account.withdraw(&dec!(1.23456789));

        assert_eq!(account.available, dec!(11.1111));
        assert_eq!(account.held, dec!(0.0000));
    }

    #[test]
    fn test_withhold_precision_to_account()
    {
        // expect back 4 d.p precision
        let mut account = Account::new();
        //assert_eq!(account.available, &dec!(0.0));
        account.deposit(&dec!(12.3456789));
        account.withhold(&dec!(1.23456789));

        assert_eq!(account.available, dec!(11.1111));
        assert_eq!(account.held, dec!(1.2346));

    }

    #[test]
    fn test_release_precision_to_account()
    {
        // expect back 4 d.p precision
        // for purposes of testing the precision
        // we are releasing half so that the 
        // calculation retrieved differs
        // even though when used it will
        // release the exact amount held 
        // (withhold) amount.
        let mut account = Account::new();
        //assert_eq!(account.available, &dec!(0.0));
        account.deposit(&dec!(12.3456789));
        account.withhold(&dec!(1.23456789));
        account.release_held(&dec!(0.61728394));

        assert_eq!(account.available, dec!(11.7284));
        assert_eq!(account.held, dec!(0.6173));

    }

    #[test]
    fn test_chargeback_precision_to_account()
    {
        // expect back 4 d.p precision
        // for purposes of testing the precision
        // we are charging half so that the 
        // calculation retrieved differs
        // even though when used it will
        // charge the exact amount held 
        // (withhold) amount.
        let mut account = Account::new();
        //assert_eq!(account.available, &dec!(0.0));
        account.deposit(&dec!(12.3456789));
        account.withhold(&dec!(1.23456789));
        account.charge(&dec!(0.61728394));

        assert_eq!(account.available, dec!(11.1111));
        assert_eq!(account.held, dec!(0.6173));

    }
}