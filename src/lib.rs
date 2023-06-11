use std::{collections::HashMap, error::Error, fs::File, io::Read};

use chrono::Duration;
use clap::{arg, value_parser, ArgMatches, Command};
use csv::{Reader, StringRecord};

fn round_to_hundredth(num: f64) -> f64 {
    (num * 100.0).round() / 100.0
}

pub fn parse_args() -> ArgMatches {
    Command::new("pint-rs")
        .version("0.1.0")
        .author("Pint-RS <@kbonnici>")
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
                .help("The GST percentage for the invoice"),
        )
        .arg(
            arg!(-f --file <FILE>)
                .required(true)
                .help("The CSV file to read from"),
        )
        .get_matches()
}
pub struct Invoice {
    time_entries: HashMap<String, f64>,
    total_time: f64,
    subtotal: f64,
    total: f64,
    pay_rate: f64,
    gst_rate: f64,
    gst: f64,
}

// implement Display for Invoice
impl std::fmt::Display for Invoice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = String::new();

        // Format the time entries
        output.push_str(&format!("{:<30} {:>10}\n", "Project", "Hours"));
        output.push_str(&format!("{:-<41}\n", ""));
        for (project, hours) in &self.time_entries {
            output.push_str(&format!("{:<30} {:>10.2}\n", project, hours));
        }
        // output.push_str(&format!("\n"));

        // Format the totals
        output.push_str(&format!(
            "\n{:<30} {:>10.2}\n\n",
            "Total Time (h)", self.total_time
        ));
        output.push_str(&format!(
            "{:<30} {:>10.2}\n",
            &format!("Subtotal at ${}/hr", self.pay_rate),
            self.subtotal
        ));
        output.push_str(&format!(
            "{:<30} {:>10.2}\n",
            &format!("GST at {}%", self.gst_rate * 100.0),
            self.gst
        ));
        output.push_str(&format!("{:<30} {:>10.2}\n", "TOTAL", self.total));

        write!(f, "{}", output)
    }
}

impl Invoice {
    fn new(pay_rate: f64, gst_rate: f64) -> Self {
        Invoice {
            time_entries: HashMap::new(),
            total_time: 0.0,
            subtotal: 0.0,
            total: 0.0,
            gst: 0.0,
            pay_rate,
            gst_rate,
        }
    }

    pub fn get_invoice(
        pay_rate: f64,
        gst: f64,
        file_name: &String,
    ) -> Result<Self, Box<dyn Error>> {
        let mut invoice = Self::new(pay_rate, gst);

        let mut file = File::open(file_name)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(contents.as_bytes());

        invoice.calculate_time_per_entry(&mut reader)?;
        invoice.calculate_invoice()?;

        Ok(invoice)
    }

    fn parse_duration(string_record: &StringRecord) -> Result<Duration, Box<dyn Error>> {
        let time_str = &string_record[3];
        let time_parts: Vec<&str> = time_str.split(':').collect();
        let hours = time_parts[0].parse::<i64>()?;
        let minutes = time_parts[1].parse::<i64>()?;
        let seconds = time_parts[2].parse::<i64>()?;
        Ok(Duration::hours(hours) + Duration::minutes(minutes) + Duration::seconds(seconds))
    }

    fn calculate_total_time(&mut self) -> Result<f64, Box<dyn Error>> {
        let mut total_time = 0.0;
        for (_, time) in self.time_entries.iter() {
            total_time += time;
        }
        Ok(round_to_hundredth(total_time))
    }

    fn calculate_time_per_entry(
        &mut self,
        reader: &mut Reader<&[u8]>,
    ) -> Result<(), Box<dyn Error>> {
        for result in reader.records() {
            let record = result?;
            let project = &record[0];

            match self.time_entries.get(project) {
                Some(time) => {
                    let duration = Self::parse_duration(&record)?;
                    self.time_entries.insert(
                        project.to_string(),
                        round_to_hundredth(time + duration.num_seconds() as f64 / 3600.0),
                    );
                }
                None => {
                    let duration = Self::parse_duration(&record)?;
                    self.time_entries.insert(
                        project.to_string(),
                        round_to_hundredth(duration.num_seconds() as f64 / 3600.0),
                    );
                }
            }
        }

        Ok(())
    }

    fn calculate_invoice(&mut self) -> Result<(), Box<dyn Error>> {
        self.total_time = self.calculate_total_time()?;

        self.subtotal = round_to_hundredth(self.total_time * self.pay_rate);
        self.gst = round_to_hundredth(self.subtotal * self.gst_rate);

        self.total = self.subtotal + (self.subtotal * self.gst_rate);

        Ok(())
    }
}
