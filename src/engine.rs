use std::error::Error;
use std::collections::HashMap;

use crate::common::*;

pub struct Engine <'a>
{
    pub accounts        : &'a mut  HashMap<u16, Account>,
    pub tx_history      : HashMap<u32, Tx>,
}

impl<'a> Engine <'a>
{
    pub fn new ( accounts : &'a mut HashMap<u16, Account>) -> Self
    {
        Engine
        {
            accounts:       accounts,
            tx_history:     HashMap::new(),
        }
    }

    pub fn read (&mut self, path : &String)
    -> Result<(), Box<dyn Error>>
    {
        csv::Reader::from_path(path)?;
        Ok(())
    }

    pub fn output (&self)
    {
        // four spaces tends to format better
        let fs = "    ";
        println!("client,{fs}available,  {fs}held, {fs}total,{fs}locked");
        for (key, val) in self.accounts.iter()
        {
            let available   = val.available;
            let held        = val.held;
            let total       = val.available + val.held;
            let locked      = val.locked;
            println!("{key},{fs}{fs}{fs}{available:.4},{fs}{held:.4},{fs}{total:.4},{fs}{locked}");
        }
    }
}