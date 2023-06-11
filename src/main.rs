use clap::Parser;
use pint_rs::{Args, Invoice};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    println!("{}", Invoice::get_invoice(&args)?);

    Ok(())
}
