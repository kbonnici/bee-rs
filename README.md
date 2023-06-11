# 🍻 Bee-rs
Author: Karl Bonnici (@kbonnici)

A command-line tool to parse Toggl reports and calculate invoice charges. Written in Rust, saving you time, allowing you to spend more of it
doing what matters 🍺

## Overview
Bee-rs is a command-line utility to parse a CSV file generated from the Toggl time-tracking app and automate the process of calculating the values 
to be inserted into an invoice, such as the following:

* Total number of hours worked
* Hours worked per project
* Subtotal
* GST applied
* Grand total

### Requirements
* A CSV file generated from the Toggl time-tracking app

### Usage
```
bee-rs --file <FILE_PATH> --gst <GST> --pay_rate <PAY_RATE>
```
* GST is a percentage value (e.g. `0.05` for 5%)
* Pay rate is an hourly rate (e.g. `50` for $50 per hour)
