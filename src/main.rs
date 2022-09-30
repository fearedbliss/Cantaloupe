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

use cantaloupe::helpers::get_source_pool_name;
use cantaloupe::providers::system;
use cantaloupe::traits::SystemProvider;
use cantaloupe::Cantaloupe;

const APP_NAME: &str = "Cantaloupe";
const APP_VERSION: &str = clap::crate_version!();
const APP_AUTHOR: &str = clap::crate_authors!();
const APP_LICENSE: &str = "Simplified BSD License";

fn print_header() {
    println!("------------------------------");
    println!("{} - {}", APP_NAME, APP_VERSION);
    println!("{}", APP_AUTHOR);
    println!("{}", APP_LICENSE);
    println!("------------------------------\n");
}

fn check_pool_imported_or_exit(system: &impl SystemProvider, backup_pool: &str) {
    if !system.is_pool_imported(backup_pool) {
        println!("{} pool is not imported. Aborting.", backup_pool);
        std::process::exit(1);
    }
}

#[derive(Parser)]
#[command(name = APP_NAME, version)]
struct Args {
    #[arg(short = 'n', long)]
    dry_run: bool,

    backup_pool: String,
    label: String,

    #[arg(num_args = 1.., required = true)]
    datasets: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let system = system::System {};

    print_header();

    let backup_pool = &args.backup_pool;
    let label = &args.label;

    // Check if the backup pool is imported.
    check_pool_imported_or_exit(&system, &backup_pool);

    let source_pools: HashSet<&str> = args
        .datasets
        .iter()
        .map(|x| get_source_pool_name(x))
        .collect();

    // Check if all of the source pools are imported.
    for source_pool in source_pools {
        if source_pool == backup_pool {
            println!("All source datasets must live outside of the backup pool. Aborting.");
            std::process::exit(1);
        }
        check_pool_imported_or_exit(&system, &source_pool);
    }

    let snapshots = system.get_all_snapshots();

    println!("Backup Pool: {}", backup_pool);
    println!("Label: {}", label);
    println!("Total Snapshots Count: {}", snapshots.len());

    for source_dataset in &args.datasets {
        println!("\n---------------");
        println!("{}", source_dataset);
        println!("---------------\n");

        let program = Cantaloupe::new(&backup_pool, &source_dataset, &label);
        let source_snapshots = program.get_source_snapshots(&snapshots);
        let backup_snapshots = program.get_backup_snapshots(&snapshots, true);

        println!("Source Snapshots Count: {}", source_snapshots.len());
        println!("Backup Snapshots Count: {}", backup_snapshots.len());

        if source_snapshots.len() == 0 {
            println!("No source snapshots available with the given dataset and label. Skipping.");
            continue;
        }

        let latest_snapshot = program.get_latest_snapshot_name(&source_snapshots);
        let common_snapshot = program.get_common_snapshot(&source_snapshots, &backup_snapshots);
        let backup_dataset = program.get_backup_dataset();

        println!("Latest Snapshot: {}", latest_snapshot);

        match common_snapshot {
            Some(common_snapshot) => {
                println!("Common Snapshot: {}", common_snapshot);

                // If we are up to date, continue.
                if common_snapshot == latest_snapshot {
                    println!("You are already up to date!");
                    continue;
                }

                // Send incremental snapshot.
                println!(
                    "Sending incremental backup for {} -> {} ...",
                    common_snapshot, latest_snapshot
                );

                if !args.dry_run {
                    let status = system.send_incremental_backup(
                        &common_snapshot,
                        &latest_snapshot,
                        &backup_dataset,
                    );

                    match status {
                        true => {
                            println!("Incremental backup finished successfully!");
                        }
                        false => {
                            println!("An error occurred while sending the incremental backup.");
                        }
                    }

                    continue;
                }
            }
            None => {
                println!("No common snapshot found.");
                println!("Sending full backup for {} ...", latest_snapshot);

                // Make sure we don't already have snapshots under this dataset since
                // we are writing to the entire dataset and want to have labeled and
                // direct incremental writes afterwards.
                let backup_snapshots = program.get_backup_snapshots(&snapshots, false);

                if backup_snapshots.len() != 0 {
                    println!("Backup pool already contains ({}) snapshots for this dataset under a different label. Skipping.", backup_snapshots.len());
                    continue;
                }

                // Create the dataset hierarchy if needed. The target backup dataset needs
                // to exist before we attempt to send into it.
                println!(
                    "Creating backup dataset hierarchy for {} (if needed) ...",
                    backup_dataset
                );

                if !args.dry_run {
                    if !system.create_dataset_tree_if_needed(&backup_dataset) {
                        println!("Failed to create backup dataset hierarchy. Perhaps your user doesn't have enough permissions for the 'zfs' command?");
                        continue;
                    }
                }

                // Doing full send.
                println!("Sending full backup for {} ...", latest_snapshot);

                if !args.dry_run {
                    let status = system.send_full_backup(&latest_snapshot, &backup_dataset);

                    match status {
                        true => {
                            println!("Full backup finished successfully!");
                        }
                        false => {
                            println!("An error occurred while sending the full backup.");
                        }
                    }

                    continue;
                }
            }
        }
    }
    println!("");
}
