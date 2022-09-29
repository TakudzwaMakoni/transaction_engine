use crate::engine::Engine;
use crate::common::*;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

impl <'a> Engine <'a>
{

    pub fn process_transactions<R>(&mut self, 
                                data    : &mut csv::Reader<R>, 
                                logger  : &mut Option<Logger>)
    -> ProcessEvent 
    where R: std::io::Read
    {
        for(entry_num, result) in data.records().enumerate()
        {
            // this shouldnt panic because we have 
            // parsed the csv file in main, and 
            // we can trust on the assumptions 
            // given by the text that the file is valid.
            let entry = result.unwrap();

            {
                let tx_type: &str = entry[0].trim();

                // these shouldn't panic since the given 
                // text allows us assume the types are valid
                let client_id: u16 = entry[1].trim().parse::<u16>().unwrap();
                let tx_id: u32 = entry[2].trim().parse::<u32>().unwrap();

                match tx_type {
                    "deposit" => 
                    {
                        // this shouldn't panic since the given 
                        // text allows us assume the types are valid
                        // however we need to round to 4 d.p.
                        let amount = Decimal::from_str(entry[3].trim())
                                     .unwrap();
                        
                        if amount < dec!(0.0000)
                        {
                            if let Some(l) = logger
                            {
                                l.log(&ProcessEvent::ErrAmountNegative(tx_id));
                            }
                            continue;
                        }

                        if let Some(_) = self.tx_history.get(&tx_id)
                        {
                            if let Some(l) = logger
                            {
                                l.log(&ProcessEvent::ErrTxIdExists(tx_id));
                            }
                            continue;
                        }

                        let account = self
                            .accounts
                            .entry(client_id)
                            .or_insert(Account::new());

                        account.deposit(&amount);

                        let tx = Tx::new(tx_id, client_id, amount, false);
                        self.tx_history.entry(tx_id).or_insert(tx);

                    }
                    "withdrawal" => 
                    {
                        // this shouldn't panic since the given 
                        // text allows us assume the types are valid
                        // however we need to round to 4 d.p.
                        let amount = Decimal::from_str(entry[3].trim())
                                     .unwrap();

                        if amount < dec!(0.0000)
                        {
                            if let Some(l) = logger
                            {
                                l.log(&ProcessEvent::ErrAmountNegative(tx_id));
                            }
                            continue;
                        }

                        if let Some(_) = self.tx_history.get(&tx_id)
                        {
                            if let Some(l) = logger
                            {
                                l.log(&ProcessEvent::ErrTxIdExists(tx_id));
                            }
                            continue;
                        }
                
                        let account = self
                            .accounts
                            .entry(client_id)
                            .or_insert(Account::new());

                        if amount > account.available 
                        {
                            if let Some(l) = logger
                            {
                                l.log(&ProcessEvent::ErrInsufficientfunds(client_id, tx_id));
                            }
                            continue;
                        }

                        account.withdraw(&amount);

                        let tx = Tx::new(tx_id, client_id, amount, false);
                        self.tx_history.entry(tx_id).or_insert(tx);
                    }
                    "dispute" => 
                    {
                        match self.tx_history.get_mut(&tx_id) 
                        {
                            Some(tx) => 
                            {
                                // this wasn't mentioned in the text
                                // since tx_ids are globally unique
                                // a client could reference a tx which
                                // is not associated with their account
                                // which shouldn't happen.
                                if tx.client != client_id
                                {
                                    // if logging enabled
                                    if let Some(l) = logger
                                    {
                                        l.log(&ProcessEvent::ErrTxNotFound(tx_id));
                                    }
                                    continue;
                                }

                                let amount = tx.amount;
                                let account = self
                                .accounts
                                .entry(client_id)
                                .or_insert(Account::new());

                                tx.disputed = true;
                                account.withhold(&amount);
                            }
                            None =>
                            {
                                // if logging enabled
                                if let Some(l) = logger
                                {
                                    l.log(&ProcessEvent::ErrTxNotFound(tx_id));
                                }
                                continue;
                            }
                        }
                    }
                    "resolve" => 
                    {
                        match self.tx_history.get_mut(&tx_id) 
                        {
                            Some(tx) =>
                            {
                                // this wasn't mentioned in the text
                                // since tx_ids are globally unique
                                // a client could reference a tx which
                                // is not associated with their account
                                // which shouldn't happen.
                                if tx.client != client_id
                                {
                                    // if logging enabled
                                    if let Some(l) = logger
                                    {
                                        l.log(&ProcessEvent::ErrTxNotFound(tx_id));
                                    }
                                    continue;
                                }

                                if !tx.disputed
                                {
                                    if let Some(l) = logger
                                    {
                                        l.log(&ProcessEvent::ErrTxNotDisputed(tx.id));
                                    }
                                    continue;
                                }
                                
                                // apply the resolve transaction.
                                let account = self.accounts.entry(client_id)
                                                  .or_insert(Account::new());
                                account.release_held(&tx.amount);
                                tx.disputed = false;
                            }
                            None =>
                            {
                                if let Some(l) = logger
                                {
                                    l.log(&ProcessEvent::ErrTxNotFound(tx_id));
                                }
                                continue;
                            }
                        }    
                    }
                    "chargeback" =>
                    {
                        match self.tx_history.get_mut(&tx_id)
                        {
                            Some(tx) =>
                            {

                                // this wasn't mentioned in the text
                                // since tx_ids are globally unique
                                // a client could reference a tx which
                                // is not associated with their account
                                // which shouldn't happen.
                                if tx.client != client_id
                                {
                                    // if logging enabled
                                    if let Some(l) = logger
                                    {
                                        l.log(&ProcessEvent::ErrUnauthorisedTx(client_id,tx_id));
                                    }
                                    continue;
                                }

                                if !tx.disputed
                                {
                                    if let Some(l) = logger
                                    {
                                        l.log(&ProcessEvent::ErrTxNotDisputed(tx.id));
                                    }
                                    continue;
                                }

                                let account = self.accounts.entry(client_id)
                                .or_insert(Account::new());
          
                                account.charge(&tx.amount);
                                account.lock();
                                tx.disputed = false;
                            }
                            None => 
                            {
                                if let Some(l) = logger
                                {
                                    l.log(&ProcessEvent::ErrTxNotFound(tx_id));
                                }
                            }
                        }
                    },
                    _ => 
                    {
                        if let Some(l) = logger
                        {
                            l.log(&ProcessEvent::ErrUnrecognisedTx(entry_num, 
                                  tx_type.to_string()));
                        }
                    }
                }
            }
        }
        ProcessEvent::ProcessComplete
    }
}

