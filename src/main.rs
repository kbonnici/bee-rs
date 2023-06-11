use std::error::Error;

use pint_rs::{parse_args, Invoice};

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args();

    let pay_rate = *args.get_one::<f64>("pay_rate").ok_or("Invalid pay rate")?;

    let file_name = args.get_one::<String>("file").ok_or("Invalid file name")?;

    let gst = *args.get_one::<f64>("gst").ok_or("Invalid gst")?;

    let invoice = Invoice::get_invoice(pay_rate, gst, &file_name)?;
    println!("{}", invoice);
    Ok(())
}
