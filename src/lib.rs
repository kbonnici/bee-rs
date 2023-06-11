use chrono::Duration;
use csv::{Reader, StringRecord};
use std::{collections::HashMap, error::Error};

use clap::Parser;
use std::path::PathBuf;

/// Generates an invoice from a CSV file
#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(
    help_template = "{about-section}\nAuthor: {author-with-newline}Version: {version}\n\n{usage-heading}\n{usage}\n\n{all-args}"
)]
pub struct Args {
    /// The pay rate for the invoice
    #[arg(short, long)]
    pay_rate: f64,

    /// The GST percentage for the invoice
    #[arg(short, long)]
    gst: Option<f64>,

    /// The CSV file to read from
    #[arg(short, long, value_name = "FILE")]
    file: PathBuf,
}

fn round_to_hundredth(num: f64) -> f64 {
    (num * 100.0).round() / 100.0
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

impl Invoice {
    fn new(args: &Args) -> Self {
        Invoice {
            time_entries: HashMap::new(),
            total_time: 0.0,
            subtotal: 0.0,
            total: 0.0,
            gst: 0.0,
            pay_rate: args.pay_rate,
            gst_rate: args.gst.unwrap_or(0.0),
        }
    }

    pub fn get_invoice(args: &Args) -> Result<Self, Box<dyn Error>> {
        let mut invoice = Self::new(args);

        let contents = std::fs::read(&args.file)?;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(contents.as_slice());

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

impl std::fmt::Display for Invoice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = String::new();

        // Format the time entries
        output.push_str(&format!("{:<30} {:>10}\n", "Project", "Hours"));
        output.push_str(&format!("{:-<41}\n", ""));
        for (project, hours) in &self.time_entries {
            output.push_str(&format!("{:<30} {:>10.2}\n", project, hours));
        }

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
