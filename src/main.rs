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

use clap::Parser;

use cantaloupe::helpers;
use cantaloupe::providers::system::System;
use cantaloupe::traits::SystemProvider;
use cantaloupe::Cantaloupe;

fn main() {
    let args = helpers::Args::parse();
    let system = System::new();

    helpers::print_header();

    let backup_pool = &args.backup_pool;
    let label = &args.label;

    // Check if the backup pool is imported.
    system.check_pool_imported_or_exit(&system, &backup_pool);

    // Check if all of the source pools are imported.
    for source_pool in helpers::get_source_pool_names(&args.datasets) {
        if source_pool == backup_pool {
            println!("All source datasets must live outside of the backup pool. Aborting.");
            std::process::exit(1);
        }
        system.check_pool_imported_or_exit(&system, &source_pool);
    }

    let snapshots = system.get_all_snapshots();

    println!("Backup Pool: {}", backup_pool);
    println!("Label: {}", label);
    println!("Total Snapshots Count: {}", snapshots.len());

    for source_dataset in &args.datasets {
        println!("\n---------------");
        println!("{}", source_dataset);
        println!("---------------\n");

        let program = Cantaloupe::new(&snapshots, &backup_pool, &source_dataset, &label);
        let source_snapshots = program.get_source_snapshots_labeled();
        let backup_snapshots = program.get_backup_snapshots_labeled();

        println!("Source Snapshots Count: {}", source_snapshots.len());
        println!("Backup Snapshots Count: {}", backup_snapshots.len());

        if source_snapshots.len() == 0 {
            println!("No source snapshots available with the given dataset and label. Skipping.");
            continue;
        }

        let latest_snapshot = program.get_latest_source_snapshot_name();
        let backup_dataset = helpers::get_backup_dataset(&backup_pool, &source_dataset);

        println!("Latest Snapshot: {}", latest_snapshot);

        if let Some(common_snapshot) = program.get_common_snapshot() {
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
                if system.send_incremental_backup(
                    &common_snapshot,
                    &latest_snapshot,
                    &backup_dataset,
                ) {
                    println!("Incremental backup finished successfully!");
                } else {
                    println!("An error occurred while sending the incremental backup.");
                }
            }
            continue;
        }

        println!("No common snapshot found.");

        // Make sure we don't already have snapshots under this dataset since
        // we are writing to the entire dataset and want to have labeled and
        // direct incremental writes afterwards.
        let backup_snapshots = program.get_backup_snapshots();

        if backup_snapshots.len() != 0 {
            println!("Backup pool already contains ({}) snapshots for this dataset under a different label. Will not do a full send. Skipping.", backup_snapshots.len());
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
            if system.send_full_backup(&latest_snapshot, &backup_dataset) {
                println!("Full backup finished successfully!");
            } else {
                println!("An error occurred while sending the full backup.");
            }
        }
        continue;
    }
    println!("");
}
