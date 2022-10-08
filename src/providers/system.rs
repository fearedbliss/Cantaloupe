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

use std::process::{Command, Stdio};

use crate::snapshot::Snapshot;
use crate::traits::SystemProvider;

pub struct System;

impl System {
    pub fn new() -> Self {
        Self {}
    }

    pub fn check_pool_imported_or_exit(&self, system: &impl SystemProvider, backup_pool: &str) {
        if !system.is_pool_imported(backup_pool) {
            println!("{} pool is not imported. Aborting.", backup_pool);
            std::process::exit(1);
        }
    }
}

impl SystemProvider for System {
    fn get_all_snapshots(&self) -> Vec<Snapshot> {
        // Example
        // -----------
        // zfs list -H -t snapshot -o name -s name
        let output = Command::new("zfs")
            .arg("list")
            .arg("-H")
            .arg("-t")
            .arg("snapshot")
            .arg("-o")
            .arg("name")
            .arg("-s")
            .arg("name")
            .output()
            .expect("failed to execute process");

        let retrieved_snapshots = String::from_utf8(output.stdout).unwrap();
        let mut unparsed_snapshots: Vec<&str> = Vec::new();

        for line in retrieved_snapshots.lines() {
            unparsed_snapshots.push(line);
        }

        Snapshot::from_batch(&unparsed_snapshots)
    }

    fn is_pool_imported(&self, pool_name: &str) -> bool {
        // Example
        // -----------
        // zpool status tank
        let status = Command::new("zpool")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("status")
            .arg(pool_name)
            .status()
            .expect("failed to execute process");

        status.success()
    }

    fn send_full_backup(&self, latest_snapshot: &str, backup_dataset: &str) -> bool {
        // Example
        // -----------
        // zfs send -p tank/ROOT/default@2022-09-27-0935-05-CHECKPOINT | zfs recv -vF backup/tank/ROOT/default
        let sender = Command::new("zfs")
            .stdout(Stdio::piped())
            .arg("send")
            .arg("-p")
            .arg(latest_snapshot)
            .spawn()
            .expect("failed to execute process");

        let receiver = Command::new("zfs")
            .stdin(sender.stdout.unwrap())
            .arg("recv")
            .arg("-vF")
            .arg(backup_dataset)
            .output()
            .expect("failed to execute process");

        receiver.status.success()
    }

    fn send_incremental_backup(
        &self,
        common_snapshot: &str,
        latest_snapshot: &str,
        backup_dataset: &str,
    ) -> bool {
        // Example
        // -----------
        // zfs send -i \
        // tank/ROOT/default@2022-09-27-0935-05-CHECKPOINT \
        // tank/ROOT/default@2022-09-28-0935-05-CHECKPOINT | \
        // zfs recv -vF backup/tank/ROOT/default
        let sender = Command::new("zfs")
            .stdout(Stdio::piped())
            .arg("send")
            .arg("-i")
            .arg(common_snapshot)
            .arg(latest_snapshot)
            .spawn()
            .expect("failed to execute process");

        let receiver = Command::new("zfs")
            .stdin(sender.stdout.unwrap())
            .arg("recv")
            .arg("-vF")
            .arg(backup_dataset)
            .output()
            .expect("failed to execute process");

        receiver.status.success()
    }

    fn create_dataset_tree_if_needed(&self, backup_dataset: &str) -> bool {
        // Example
        // -----------
        // zfs create -p backup/tank/ROOT
        let status = Command::new("zfs")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("create")
            .arg("-p")
            .arg(backup_dataset)
            .status()
            .expect("failed to execute process");

        status.success()
    }
}
