use clap::Parser;
use pint_rs::{Args, Invoice, InvoiceBuilder};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let invoice: Invoice = InvoiceBuilder::new(&args).import_csv(&args.file)?.build();

    println!("{}", invoice);

    Ok(())
}
