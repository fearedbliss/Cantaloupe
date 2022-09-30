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

use std::fmt::{Display, Formatter, Result};

#[derive(Eq, Ord, PartialOrd, Clone, Debug)]
pub struct Snapshot {
    pub name: String,
}

impl Snapshot {
    pub fn new(name: &str) -> Snapshot {
        Snapshot {
            name: String::from(name),
        }
    }

    pub fn from_batch(unparsed_snapshots: &Vec<&str>) -> Vec<Snapshot> {
        let mut snapshots = Vec::new();

        for snapshot in unparsed_snapshots {
            if !Snapshot::validate_snapshot_format(&snapshot) {
                continue;
            }

            snapshots.push(Snapshot::new(snapshot));
        }

        snapshots.sort();
        snapshots
    }

    fn validate_snapshot_format(name: &str) -> bool {
        let core_splinters: Vec<_> = name.split("@").collect();
        let splinters: Vec<_> = core_splinters[1].split("-").collect();

        let year = splinters.get(0);
        let month = splinters.get(1);
        let day = splinters.get(2);
        let hour = splinters.get(3);
        let seconds = splinters.get(4);
        let label = splinters.get(5);

        if year == None
            || month == None
            || day == None
            || hour == None
            || seconds == None
            || label == None
        {
            return false;
        }

        true
    }
}

impl PartialEq for Snapshot {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Display for Snapshot {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_batch_format_should_return_snapshots() {
        let unparsed_snapshots = vec![
            "tank@lol",
            "tank@2022-09-27-1300-00-TEST",
            "tank@ok",
            "tank@2022-05-10-0000-00-TEST",
        ];
        let expected_snapshots = vec![
            Snapshot::new("tank@2022-05-10-0000-00-TEST"),
            Snapshot::new("tank@2022-09-27-1300-00-TEST"),
        ];

        let snapshots = Snapshot::from_batch(&unparsed_snapshots);

        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots, expected_snapshots);
    }

    #[test]
    fn test_from_batch_format_should_not_return_snapshots() {
        let unparsed_snapshots = vec![
            "tank@lol",
            "tank@2022-09-27-1300-MISTAKE",
            "tank@ok",
            "tank@nothing",
        ];

        let snapshots = Snapshot::from_batch(&unparsed_snapshots);

        assert_eq!(snapshots.len(), 0);
    }
}
