extern crate tx_engine;

use std::process;
use std::env;
use std::collections::HashMap;

use tx_engine::common::Logger;
use tx_engine::common::Account;
use tx_engine::common::ProcessEvent;

fn the_app()
-> ProcessEvent
{
    // get required command line argument
    let args : Vec<String> = env::args().collect();
    if args.len() < 2
    {
        println!("usage:\n cargo run -- [transactions file] [(OPTIONAL) log file]");
        process::exit(1);
    }

    // setup optional logger
    let mut logger : Option<Logger> = None;
    if args.len() == 3
    {
        logger = Logger::new(&args[2]);
    }

    // setup csv data
    let mut data = match csv::Reader::from_path(&args[1])
    {
        Ok(d)=> d,
        Err(err)=>
        {
            return 
            ProcessEvent::ExternalErr(err.to_string())
        }
    };

    let mut accounts : HashMap<u16, Account> = HashMap::new();
    let mut engine = tx_engine::engine::Engine::new(&mut accounts);
    engine.process_transactions(&mut data,&mut logger);
    engine.output();

    ProcessEvent::ProcessComplete
}

fn main()
{

    match the_app()
    {
        ProcessEvent::ProcessComplete =>{}
        ProcessEvent::ExternalErr(err)=>
        {
            println!("App failed: {err}");
            process::exit(1);
        }
        _ =>{}
    }
}