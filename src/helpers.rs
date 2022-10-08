// Copyright © 2022 Jonathan Vasquez <jon@xyinn.org>
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
// OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
// HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
// LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
// OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGE.

use std::collections::HashSet;

use clap::Parser;

const APP_NAME: &str = "Cantaloupe";
const APP_VERSION: &str = clap::crate_version!();
const APP_AUTHOR: &str = clap::crate_authors!();
const APP_LICENSE: &str = "Simplified BSD License";

pub fn print_header() {
    println!("------------------------------");
    println!("{} - {}", APP_NAME, APP_VERSION);
    println!("{}", APP_AUTHOR);
    println!("{}", APP_LICENSE);
    println!("------------------------------\n");
}

#[derive(Parser)]
#[command(name = APP_NAME, version)]
pub struct Args {
    #[arg(
        short = 'n',
        long,
        help = "Performs a dry run. Does not require root privileges."
    )]
    pub dry_run: bool,

    pub backup_pool: String,
    pub label: String,

    #[arg(num_args = 1.., required = true)]
    pub datasets: Vec<String>,
}

pub fn get_source_pool_name(dataset_name: &str) -> &str {
    let splinters: Vec<_> = dataset_name.split("/").collect();
    splinters[0]
}

pub fn get_source_pool_names(datasets: &Vec<String>) -> HashSet<&str> {
    datasets.iter().map(|x| get_source_pool_name(x)).collect()
}

pub fn get_backup_dataset(backup_pool_name: &str, source_dataset_name: &str) -> String {
    format!("{}/{}", backup_pool_name, source_dataset_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_source_pool_name_should_return_pool_name() {
        let source_pool_name = get_source_pool_name("tank/var/log");

        assert_eq!(source_pool_name, "tank");
    }

    #[test]
    fn test_get_source_pool_names_should_be_correct() {
        let source_datasets = vec![
            String::from("tank/var/log"),
            String::from("elephants/in/space"),
            String::from("dolphins/are/awesome"),
        ];
        let expected = HashSet::from(["tank", "elephants", "dolphins"]);

        let result = get_source_pool_names(&source_datasets);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_backup_dataset_should_get_name() {
        let expected_name = "backup/tank/var/log";

        let name = get_backup_dataset("backup", "tank/var/log");

        assert_eq!(name, expected_name);
    }
}
