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

#![allow(unused_variables)]
use crate::snapshot::Snapshot;
use crate::traits::SystemProvider;

// Not currently used. It was before when I needed
// to inject this dependency for testing into the core
// system. Leaving here for now.
pub struct FakeSystem {
    pub snapshots: Vec<Snapshot>,
}

impl FakeSystem {
    pub fn new(snapshots: Vec<Snapshot>) -> FakeSystem {
        FakeSystem { snapshots }
    }
}

impl SystemProvider for FakeSystem {
    fn get_all_snapshots(&self) -> Vec<Snapshot> {
        self.snapshots.clone()
    }

    fn is_pool_imported(&self, pool_name: &str) -> bool {
        true
    }

    fn send_full_backup(&self, latest_snapshot: &str, backup_dataset: &str) -> bool {
        true
    }

    fn send_incremental_backup(
        &self,
        ancestor_snapshot: &str,
        latest_snapshot: &str,
        backup_dataset: &str,
    ) -> bool {
        true
    }

    fn create_dataset_tree_if_needed(&self, backup_dataset: &str) -> bool {
        true
    }
}
