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

pub mod helpers;
pub mod providers;
pub mod snapshot;
pub mod testing;
pub mod traits;

use crate::snapshot::Snapshot;

pub struct Cantaloupe {
    snapshots: Vec<Snapshot>,
    backup_pool_name: String,
    backup_dataset_name: String,
    label: String,
    source_snapshots_labeled: Vec<Snapshot>,
    backup_snapshots_labeled: Vec<Snapshot>,
}

impl Cantaloupe {
    pub fn new(
        pending_snapshots: &Vec<Snapshot>,
        backup_pool_name: &str,
        source_dataset_name: &str,
        label: &str,
    ) -> Self {
        let mut snapshots = pending_snapshots.clone();
        snapshots.sort_unstable();

        let backup_dataset_name =
            helpers::get_backup_dataset(backup_pool_name, source_dataset_name);
        let source_snapshots_labeled =
            Self::get_snapshots(&snapshots, &source_dataset_name, &label, true);
        let backup_snapshots_labeled =
            Self::get_snapshots(&snapshots, &backup_dataset_name, &label, true);

        Self {
            snapshots,
            backup_pool_name: String::from(backup_pool_name),
            backup_dataset_name: backup_dataset_name,
            label: String::from(label),
            source_snapshots_labeled,
            backup_snapshots_labeled,
        }
    }

    // Gets all the source snapshots matching self.label.
    pub fn get_source_snapshots_labeled(&self) -> &Vec<Snapshot> {
        &self.source_snapshots_labeled
    }

    // Gets all the backup snapshots matching self.label.
    pub fn get_backup_snapshots_labeled(&self) -> &Vec<Snapshot> {
        &self.backup_snapshots_labeled
    }

    // Gets all the backup snapshots (label ignored).
    pub fn get_backup_snapshots(&self) -> Vec<Snapshot> {
        Self::get_snapshots(
            &self.snapshots,
            &self.backup_dataset_name,
            &self.label,
            false,
        )
    }

    pub fn get_common_snapshot(&self) -> Option<&str> {
        for source_snapshot in self.source_snapshots_labeled.iter().rev() {
            let backup_snapshot = format!("{}/{}", self.backup_pool_name, source_snapshot);
            if self
                .backup_snapshots_labeled
                .contains(&&Snapshot::new(&backup_snapshot))
            {
                return Some(&source_snapshot.name);
            }
        }
        None
    }

    pub fn get_latest_source_snapshot_name(&self) -> &str {
        self.source_snapshots_labeled.last().unwrap().name.as_str()
    }

    fn get_snapshots(
        snapshots: &Vec<Snapshot>,
        dataset_name: &str,
        label: &str,
        use_label: bool,
    ) -> Vec<Snapshot> {
        let mut filtered_snapshots: Vec<Snapshot> = Vec::new();
        let prefix = format!("{}@", dataset_name);
        let suffix = format!("-{}", label);
        for snapshot in snapshots {
            if snapshot.name.starts_with(&prefix) {
                if use_label {
                    if snapshot.name.ends_with(&suffix) {
                        filtered_snapshots.push(snapshot.clone());
                    }
                } else {
                    filtered_snapshots.push(snapshot.clone());
                }
            }
        }
        filtered_snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_example_snapshots() -> Vec<Snapshot> {
        vec![
            Snapshot::new("tank/var/log@2021-06-03-1800-00-TEST"),
            Snapshot::new("tank/var/log@2021-01-01-1300-12-TEST"),
            Snapshot::new("tank/var/log@2022-10-05-1953-12-TEST"),
            Snapshot::new("tank/var/log@2022-09-29-1512-00-CHECKPOINT"),
            Snapshot::new("tank/usr/home@2022-09-29-1512-00-ELEPHANT"),
            Snapshot::new("backup/tank/var/log@2020-05-13-0013-23-TEST"),
            Snapshot::new("backup/tank/var/log@2021-07-23-0548-19-LOL"),
            Snapshot::new("backup/tank/var/log@2021-06-03-1800-00-TEST"),
            Snapshot::new("backup/usr/ports@2021-06-03-1800-00-DOLPHIN"),
        ]
    }

    fn get_example_snapshots_no_common_snapshots() -> Vec<Snapshot> {
        vec![
            Snapshot::new("tank/var/log@2021-06-03-1800-00-TEST"),
            Snapshot::new("backup/tank/var/log@2021-07-23-0548-19-LOL"),
        ]
    }

    #[test]
    fn test_get_source_snapshots_labeled_should_return_snapshots_with_correct_label() {
        let program = Cantaloupe::new(&get_example_snapshots(), "backup", "tank/var/log", "TEST");
        let expected_snapshots = vec![
            Snapshot::new("tank/var/log@2021-01-01-1300-12-TEST"),
            Snapshot::new("tank/var/log@2021-06-03-1800-00-TEST"),
            Snapshot::new("tank/var/log@2022-10-05-1953-12-TEST"),
        ];

        let snapshots = program.get_source_snapshots_labeled();

        assert_eq!(snapshots.len(), 3);

        for expected_snapshot in expected_snapshots {
            assert!(snapshots.contains(&&expected_snapshot));
        }
    }

    #[test]
    fn test_get_backup_snapshots_labeled_should_return_snapshots_with_correct_label() {
        let program = Cantaloupe::new(&get_example_snapshots(), "backup", "tank/var/log", "TEST");
        let expected_snapshots = vec![
            Snapshot::new("backup/tank/var/log@2020-05-13-0013-23-TEST"),
            Snapshot::new("backup/tank/var/log@2021-06-03-1800-00-TEST"),
        ];

        let snapshots = program.get_backup_snapshots_labeled();

        assert_eq!(snapshots.len(), 2);

        for expected_snapshot in expected_snapshots {
            assert!(snapshots.contains(&&expected_snapshot));
        }
    }

    #[test]
    fn test_get_backup_snapshots_should_get_all_backup_snapshots() {
        let program = Cantaloupe::new(&get_example_snapshots(), "backup", "tank/var/log", "TEST");
        let expected_snapshots = vec![
            Snapshot::new("backup/tank/var/log@2020-05-13-0013-23-TEST"),
            Snapshot::new("backup/tank/var/log@2021-07-23-0548-19-LOL"),
            Snapshot::new("backup/tank/var/log@2021-06-03-1800-00-TEST"),
        ];

        let snapshots = program.get_backup_snapshots();

        assert_eq!(snapshots.len(), 3);

        for expected_snapshot in expected_snapshots {
            assert!(snapshots.contains(&&expected_snapshot));
        }
    }

    #[test]
    fn test_get_common_snapshot_should_return_common_snapshot() {
        let program = Cantaloupe::new(&get_example_snapshots(), "backup", "tank/var/log", "TEST");
        let expected_common_snapshot = "tank/var/log@2021-06-03-1800-00-TEST";

        let common_snapshot = program.get_common_snapshot();

        assert_eq!(common_snapshot.unwrap(), expected_common_snapshot);
    }

    #[test]
    fn test_get_common_snapshot_should_not_return_common_snapshot() {
        let program = Cantaloupe::new(
            &get_example_snapshots_no_common_snapshots(),
            "backup",
            "tank/var/log",
            "TEST",
        );

        let common_snapshot = program.get_common_snapshot();

        assert!(common_snapshot.is_none());
    }

    #[test]
    fn test_get_latest_source_snapshot_name_should_get_latest() {
        let snapshots = vec![
            Snapshot::new("tank/var/log@2021-07-23-0548-19-TEST"),
            Snapshot::new("tank/var/log@2020-05-13-0013-23-TEST"),
            Snapshot::new("tank/var/log@2021-06-03-1800-00-TEST"),
            Snapshot::new("backup/tank/var/log@2022-12-10-1800-00-TEST"),
            Snapshot::new("zebra/tank/var/log@2022-12-10-1800-00-TEST"),
        ];
        let expected_snapshot = "tank/var/log@2021-07-23-0548-19-TEST";
        let program = Cantaloupe::new(&snapshots, "backup", "tank/var/log", "TEST");

        let snapshot = program.get_latest_source_snapshot_name();

        assert_eq!(snapshot, expected_snapshot);
    }
}
