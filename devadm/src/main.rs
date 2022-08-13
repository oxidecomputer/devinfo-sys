// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2022 Oxide Computer Company

use anyhow::Result;
use clap::{AppSettings, Parser};
use colored::*;
use devinfo::get_devices;
use std::io::{stdout, Write};
use tabwriter::TabWriter;

#[derive(Parser)]
#[clap(
    version = "0.1",
    author = "Ryan Goodfellow <ryan.goodfellow@oxide.computer>"
)]
#[clap(setting = AppSettings::InferSubcommands)]
struct Opts {
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// Show device information. All numeric values in hex.
    Show(Show),
}

struct I32(i32);

impl std::str::FromStr for I32 {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(suffix) = s.strip_prefix("0x") {
            Ok(I32(i32::from_str_radix(suffix, 16)?))
        } else {
            Ok(I32(i32::from_str_radix(s, 16)?))
        }
    }
}

#[derive(Parser)]
#[clap(setting = AppSettings::InferSubcommands)]
struct Show {
    /// Filter by device name.
    filter: Option<String>,

    /// Filter by device id (hex values only).
    #[clap(short, long)]
    id: Option<I32>,

    /// Filter by device vendor (hex values only).
    #[clap(short, long)]
    vendor: Option<I32>,

    /// Fetch device prom data (requires root privilege)
    #[clap(short, long)]
    prom: bool,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Show(ref s) => show_devices(&opts, s),
    }
}

fn show_devices(_opts: &Opts, s: &Show) -> Result<()> {
    let info = get_devices(s.prom)?;

    for (name, dev_info) in info {
        match &s.filter {
            Some(f) => {
                if !name.eq(f) {
                    continue;
                }
            }
            None => {}
        }

        match &s.id {
            Some(id) => match dev_info.props.get("device-id") {
                Some(value) => {
                    if !value.matches_int(id.0) {
                        continue;
                    }
                }
                None => {
                    continue;
                }
            },
            None => {}
        }

        match &s.vendor {
            Some(vendor) => match dev_info.props.get("vendor-id") {
                Some(value) => {
                    if !value.matches_int(vendor.0) {
                        continue;
                    }
                }
                None => {
                    continue;
                }
            },
            None => {}
        }

        println!("{}", name.bright_blue().bold());
        println!("{}", "=".repeat(name.len()).bright_black());

        let mut tw = TabWriter::new(stdout());
        writeln!(&mut tw, "{}\t{}", "property".dimmed(), "value".dimmed())?;
        writeln!(
            &mut tw,
            "{}\t{}",
            "--------".bright_black(),
            "-----".bright_black(),
        )?;
        for (prop_name, value) in dev_info.props {
            writeln!(&mut tw, "{}\t{}", prop_name, format!("{}", value))?;
        }
        tw.flush()?;
        println!();
    }

    Ok(())
}
