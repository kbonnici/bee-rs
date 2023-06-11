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
    calculated_total_time: f64,
    calculated_subtotal: f64,
    calculated_total: f64,
    calculated_gst: f64,
    pay_rate: f64,
    gst_rate: f64,
}

impl Invoice {
    fn new(args: &Args) -> Self {
        Invoice {
            time_entries: HashMap::new(),
            calculated_total_time: 0.0,
            calculated_subtotal: 0.0,
            calculated_total: 0.0,
            calculated_gst: 0.0,
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

        let hours: i64 = time_parts[0].parse()?;
        let minutes: i64 = time_parts[1].parse()?;
        let seconds: i64 = time_parts[2].parse()?;

        let duration =
            Duration::hours(hours) + Duration::minutes(minutes) + Duration::seconds(seconds);

        Ok(duration)
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
            let duration = Self::parse_duration(&record)?;

            if let Some(time) = self.time_entries.get_mut(project) {
                *time += round_to_hundredth(duration.num_seconds() as f64 / 3600.0)
            } else {
                self.time_entries.insert(
                    project.to_string(),
                    round_to_hundredth(duration.num_seconds() as f64 / 3600.0),
                );
            }
        }

        Ok(())
    }

    fn calculate_invoice(&mut self) -> Result<(), Box<dyn Error>> {
        self.calculated_total_time = self.calculate_total_time()?;

        self.calculated_subtotal = round_to_hundredth(self.calculated_total_time * self.pay_rate);
        self.calculated_gst = round_to_hundredth(self.calculated_subtotal * self.gst_rate);
        self.calculated_total =
            self.calculated_subtotal + (self.calculated_subtotal * self.gst_rate);

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
            "Total Time (h)", self.calculated_total_time
        ));
        output.push_str(&format!(
            "{:<30} {:>10.2}\n",
            &format!("Subtotal at ${}/hr", self.pay_rate),
            self.calculated_subtotal
        ));
        output.push_str(&format!(
            "{:<30} {:>10.2}\n",
            &format!("GST at {}%", self.gst_rate * 100.0),
            self.calculated_gst
        ));
        output.push_str(&format!(
            "{:<30} {:>10.2}\n",
            "TOTAL", self.calculated_total
        ));

        write!(f, "{}", output)
    }
}
