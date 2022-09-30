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
    backup_pool_name: String,
    source_dataset_name: String,
    label: String,
}

impl Cantaloupe {
    pub fn new(backup_pool_name: &str, source_dataset_name: &str, label: &str) -> Self {
        Self {
            backup_pool_name: String::from(backup_pool_name),
            source_dataset_name: String::from(source_dataset_name),
            label: String::from(label),
        }
    }

    pub fn get_source_snapshots<'a>(&self, snapshots: &'a Vec<Snapshot>) -> Vec<&'a Snapshot> {
        self.get_snapshots(&self.source_dataset_name, &self.label, &snapshots, true)
    }

    pub fn get_backup_snapshots<'a>(
        &self,
        snapshots: &'a Vec<Snapshot>,
        use_label: bool,
    ) -> Vec<&'a Snapshot> {
        let backup_dataset = self.get_backup_dataset();
        self.get_snapshots(&backup_dataset, &self.label, &snapshots, use_label)
    }

    pub fn get_common_snapshot<'a>(
        &'a self,
        source_snapshots: &Vec<&'a Snapshot>,
        backup_snapshots: &Vec<&Snapshot>,
    ) -> Option<&str> {
        for source_snapshot in source_snapshots.iter().rev() {
            let backup_snapshot = format!("{}/{}", self.backup_pool_name, source_snapshot);
            if backup_snapshots.contains(&&Snapshot::new(&backup_snapshot)) {
                return Some(&source_snapshot.name);
            }
        }
        None
    }

    pub fn get_latest_snapshot_name<'a>(&self, snapshots: &'a Vec<&Snapshot>) -> &'a str {
        snapshots.last().unwrap().name.as_str()
    }

    pub fn get_backup_dataset(&self) -> String {
        format!("{}/{}", self.backup_pool_name, self.source_dataset_name)
    }

    fn get_snapshots<'a>(
        &self,
        dataset_name: &str,
        label: &str,
        snapshots: &'a Vec<Snapshot>,
        use_label: bool,
    ) -> Vec<&'a Snapshot> {
        let mut filtered_snapshots: Vec<&Snapshot> = Vec::new();
        let prefix = format!("{}@", dataset_name);
        let suffix = format!("-{}", label);
        for snapshot in snapshots {
            if snapshot.name.starts_with(&prefix) {
                if use_label {
                    if snapshot.name.ends_with(&suffix) {
                        filtered_snapshots.push(snapshot);
                    }
                } else {
                    filtered_snapshots.push(snapshot);
                }
            }
        }
        filtered_snapshots.sort();
        filtered_snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_example_snapshots() -> Vec<Snapshot> {
        let mut snapshots = Vec::new();
        snapshots.push(Snapshot::new("tank/var/log@2021-06-03-1800-00-TEST"));
        snapshots.push(Snapshot::new("tank/var/log@2021-01-01-1300-12-TEST"));
        snapshots.push(Snapshot::new("tank/var/log@2022-10-05-1953-12-TEST"));
        snapshots.push(Snapshot::new("tank/var/log@2022-09-29-1512-00-CHECKPOINT"));
        snapshots.push(Snapshot::new("backup/tank/var/log@2020-05-13-0013-23-TEST"));
        snapshots.push(Snapshot::new("backup/tank/var/log@2021-07-23-0548-19-LOL"));
        snapshots.push(Snapshot::new("backup/tank/var/log@2021-06-03-1800-00-TEST"));
        snapshots
    }

    #[test]
    fn test_get_source_snapshots_should_return_snapshots_with_correct_label() {
        let all_snapshots = get_example_snapshots();
        let program = Cantaloupe::new("backup", "tank/var/log", "TEST");
        let expected_snapshots = vec![
            Snapshot::new("tank/var/log@2021-01-01-1300-12-TEST"),
            Snapshot::new("tank/var/log@2021-06-03-1800-00-TEST"),
            Snapshot::new("tank/var/log@2022-10-05-1953-12-TEST"),
        ];

        let snapshots = program.get_source_snapshots(&all_snapshots);

        assert_eq!(snapshots.len(), 3);

        for expected_snapshot in expected_snapshots {
            assert!(snapshots.contains(&&expected_snapshot));
        }
    }

    #[test]
    fn test_get_backup_snapshots_should_return_snapshots_with_correct_label() {
        let all_snapshots = get_example_snapshots();
        let program = Cantaloupe::new("backup", "tank/var/log", "TEST");
        let expected_snapshots = vec![
            Snapshot::new("backup/tank/var/log@2020-05-13-0013-23-TEST"),
            Snapshot::new("backup/tank/var/log@2021-06-03-1800-00-TEST"),
        ];

        let snapshots = program.get_backup_snapshots(&all_snapshots, true);

        assert_eq!(snapshots.len(), 2);

        for expected_snapshot in expected_snapshots {
            assert!(snapshots.contains(&&expected_snapshot));
        }
    }

    #[test]
    fn test_get_backup_snapshots_should_get_all_snapshots_and_not_use_label() {
        let all_snapshots = get_example_snapshots();
        let program = Cantaloupe::new("backup", "tank/var/log", "TEST");
        let expected_snapshots = vec![
            Snapshot::new("backup/tank/var/log@2020-05-13-0013-23-TEST"),
            Snapshot::new("backup/tank/var/log@2021-07-23-0548-19-LOL"),
            Snapshot::new("backup/tank/var/log@2021-06-03-1800-00-TEST"),
        ];

        let snapshots = program.get_backup_snapshots(&all_snapshots, false);

        assert_eq!(snapshots.len(), 3);

        for expected_snapshot in expected_snapshots {
            assert!(snapshots.contains(&&expected_snapshot));
        }
    }

    #[test]
    fn test_get_common_snapshot_should_return_common_snapshot() {
        let all_snapshots = get_example_snapshots();
        let program = Cantaloupe::new("backup", "tank/var/log", "TEST");

        let source_snapshots = program.get_source_snapshots(&all_snapshots);
        let backup_snapshots = program.get_backup_snapshots(&all_snapshots, true);
        let expected_common_snapshot = "tank/var/log@2021-06-03-1800-00-TEST";

        let common_snapshot = program.get_common_snapshot(&source_snapshots, &backup_snapshots);

        assert_eq!(common_snapshot.unwrap(), expected_common_snapshot);
    }

    #[test]
    fn test_get_common_snapshot_should_not_return_common_snapshot() {
        let all_snapshots = get_example_snapshots();
        let program = Cantaloupe::new("backup", "tank/var/log", "TEST");

        let temporary_snapshot = Snapshot::new("backup/tank/var/log@2022-01-01-0000-00-TEST");
        let source_snapshots = program.get_source_snapshots(&all_snapshots);
        let backup_snapshots = vec![&temporary_snapshot];

        let common_snapshot = program.get_common_snapshot(&source_snapshots, &backup_snapshots);

        assert!(common_snapshot.is_none());
    }

    #[test]
    fn test_get_latest_snapshot_name_should_get_latest() {
        let snapshots = vec![
            Snapshot::new("tank/var/log@2021-07-23-0548-19-TEST"),
            Snapshot::new("tank/var/log@2020-05-13-0013-23-TEST"),
            Snapshot::new("tank/var/log@2021-06-03-1800-00-TEST"),
        ];

        let expected_snapshot = "tank/var/log@2021-07-23-0548-19-TEST";
        let program = Cantaloupe::new("backup", "tank/var/log", "TEST");
        let references = program.get_source_snapshots(&snapshots);

        let snapshot = program.get_latest_snapshot_name(&references);

        assert_eq!(snapshot, expected_snapshot);
    }

    #[test]
    fn test_get_backup_dataset_should_get_name() {
        let program = Cantaloupe::new("backup", "tank/var/log", "TEST");
        let expected_name = "backup/tank/var/log";

        let name = program.get_backup_dataset();

        assert_eq!(name, expected_name);
    }
}
