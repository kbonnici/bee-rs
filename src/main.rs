use std::error::Error;

use bee_rs::Invoice;
use clap::{arg, value_parser, Command};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("bee-rs")
        .version("0.1.0")
        .author("Bee-RS <@kbonnici>")
        .about("Generates an invoice from a CSV file")
        .arg(
            arg!(-p --pay_rate <VALUE>)
                .required(true)
                .value_parser(value_parser!(f64))
                .help("The pay rate for the invoice"),
        )
        .arg(
            arg!(-g --gst <VALUE>)
                .required(false)
                .default_value("0.05")
                .value_parser(value_parser!(f64))
                .help("The pay rate for the invoice"),
        )
        .arg(
            arg!(-f --file <FILE>)
                .required(true)
                .help("The CSV file to read from"),
        )
        .get_matches();

    let pay_rate = *matches
        .get_one::<f64>("pay_rate")
        .ok_or("Invalid pay rate")?;

    let file_name = matches
        .get_one::<String>("file")
        .ok_or("Invalid file name")?;

    let gst = *matches.get_one::<f64>("gst").ok_or("Invalid gst")?;

    let invoice = Invoice::get_invoice(pay_rate, gst, &file_name)?;
    println!("{}", invoice);
    Ok(())
}
