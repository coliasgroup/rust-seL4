//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use anyhow::Result;
use clap::{Arg, ArgAction, Command};

#[derive(Debug)]
pub struct Args {
    pub in_file_path: String,
    pub out_file_path: String,
    pub verbose: bool,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let matches = Command::new("")
            .arg(Arg::new("in_file").value_name("IN"))
            .arg(
                Arg::new("out_file")
                    .short('o')
                    .value_name("OUT")
                    .required(true),
            )
            .arg(Arg::new("verbose").short('v').action(ArgAction::SetTrue))
            .get_matches();

        let in_file_path = matches.get_one::<String>("in_file").unwrap().to_owned();
        let out_file_path = matches.get_one::<String>("out_file").unwrap().to_owned();
        let verbose = *matches.get_one::<bool>("verbose").unwrap();

        Ok(Self {
            in_file_path,
            out_file_path,
            verbose,
        })
    }
}
