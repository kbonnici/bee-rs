use anyhow::{Context, Result};
use chrono::Duration;
use clap::Parser;
use csv::Reader;
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(test)]
mod tests;

/// Generates an invoice from a CSV file
#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(
    help_template = "{about-section}\nAuthor: {author-with-newline}Version: {version}\n\n{usage-heading}\n{usage}\n\n{all-args}"
)]
pub struct Args {
    /// The pay rate for the invoice
    #[arg(short, long)]
    pub pay_rate: f64,

    /// The GST percentage for the invoice
    #[arg(short, long)]
    pub gst: Option<f64>,

    /// The CSV file to read from
    #[arg(short, long, value_name = "FILE")]
    pub file: PathBuf,
}

fn round_to_hundredth(num: f64) -> f64 {
    (num * 100.0).round() / 100.0
}

#[derive(Debug, PartialEq)]
pub struct InvoiceBuilder {
    project_hours_logged: HashMap<String, f64>,
    pay_rate: f64,
    gst_rate: f64,
}

#[derive(Debug, PartialEq)]
pub struct Invoice {
    project_hours_logged: HashMap<String, f64>,
    total_time: f64,
    subtotal: f64,
    gst: f64,
    total: f64,

    gst_rate: f64,
    pay_rate: f64,
}

impl InvoiceBuilder {
    pub fn new(args: &Args) -> Self {
        Self {
            project_hours_logged: HashMap::new(),
            pay_rate: args.pay_rate,
            gst_rate: args.gst.unwrap_or(0.0),
        }
    }

    pub fn build(&self) -> Invoice {
        let total_time = round_to_hundredth(self.project_hours_logged.iter().map(|(_, t)| t).sum());

        let subtotal = round_to_hundredth(total_time * self.pay_rate);
        let gst = round_to_hundredth(subtotal * self.gst_rate);

        let total = subtotal + gst;

        Invoice {
            project_hours_logged: self.project_hours_logged.clone(),
            total_time,
            subtotal,
            gst,
            total,

            gst_rate: self.gst_rate,
            pay_rate: self.pay_rate,
        }
    }

    pub fn add_project_duration(&mut self, project: &str, duration: &Duration) -> &mut Self {
        if let Some(time) = self.project_hours_logged.get_mut(project) {
            *time += round_to_hundredth(duration.num_seconds() as f64 / 3600.0)
        } else {
            self.project_hours_logged.insert(
                project.to_owned(),
                round_to_hundredth(duration.num_seconds() as f64 / 3600.0),
            );
        }

        self
    }

    pub fn collect_time_entries(&mut self, entries: &[(String, Duration)]) -> &mut Self {
        for (project, duration) in entries {
            self.add_project_duration(&project, duration);
        }

        self
    }

    pub fn import_csv(&mut self, file: &PathBuf) -> Result<&mut Self> {
        let contents = std::fs::read(file)
            .with_context(|| format!("Unable to read from given file \"{:?}\"", file))?;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(contents.as_slice());

        let entries =
            Self::parse_csv_entries(&mut reader).context("Unable to parse CSV entries")?;
        self.collect_time_entries(&entries);

        Ok(self)
    }

    fn parse_duration_str(str: &str) -> Result<Duration> {
        let time_parts: Vec<&str> = str.split(':').collect();

        let hours: i64 = time_parts[0]
            .parse()
            .with_context(|| format!("Unable to parse hour string {}", time_parts[0]))?;
        let minutes: i64 = time_parts[1]
            .parse()
            .with_context(|| format!("Unable to parse minutes string {}", time_parts[1]))?;
        let seconds: i64 = time_parts[2]
            .parse()
            .with_context(|| format!("Unable to parse seconds string {}", time_parts[2]))?;

        let duration =
            Duration::hours(hours) + Duration::minutes(minutes) + Duration::seconds(seconds);

        Ok(duration)
    }

    fn parse_csv_entries(reader: &mut Reader<&[u8]>) -> Result<Vec<(String, Duration)>> {
        let entries: Vec<(String, Duration)> = reader
            .records()
            .filter_map(|r| r.ok())
            .flat_map(|r| {
                Ok::<(String, Duration), anyhow::Error>((
                    r[0].to_owned(),
                    Self::parse_duration_str(&r[3])
                        .with_context(|| format!("Unable to parse duration {}", &r[3]))?,
                ))
            })
            .collect();

        Ok(entries)
    }
}

impl std::fmt::Display for Invoice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = String::new();

        // Format the time entries
        output.push_str(&format!("{:<30} {:>10}\n", "Project", "Hours"));
        output.push_str(&format!("{:-<41}\n", ""));
        for (project, hours) in &self.project_hours_logged {
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
