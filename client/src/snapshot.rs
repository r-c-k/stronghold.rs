// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use engine::{
    snapshot::{decrypt_snapshot, encrypt_snapshot, snapshot_dir},
    vault::BoxProvider,
};

use std::{fs::OpenOptions, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub state: Vec<u8>,
}

impl Snapshot {
    pub fn new<P>(state: Vec<u8>) -> Self
where {
        Self { state }
    }

    pub fn get_state(self) -> Vec<u8> {
        self.state
    }

    pub fn get_snapshot_path() -> PathBuf {
        let path = snapshot_dir().expect("Unable to get the snapshot directory");
        path.join("backup.snapshot")
    }
    pub fn read_from_snapshot<P>(snapshot: &PathBuf, pass: &str) -> Self
    where
        P: BoxProvider + Clone + Send + Sync,
    {
        let mut buffer = Vec::new();
        let mut file = OpenOptions::new()
            .read(true)
            .open(snapshot)
            .expect("Unable to access snapshot. Make sure that it exists or run encrypt to build a new one.");
        decrypt_snapshot(&mut file, &mut buffer, pass.as_bytes()).expect("unable to decrypt the snapshot");

        Snapshot::new::<P>(buffer)
    }
    pub fn write_to_snapshot(self, snapshot: &PathBuf, pass: &str) {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(snapshot)
            .expect("Unable to access snapshot. Make sure that it exists or run encrypt to build a new one.");
        // clear contents of the file before writing.
        file.set_len(0).expect("unable to clear the contents of the file file");
        encrypt_snapshot(self.state, &mut file, pass.as_bytes()).expect("Couldn't write to the snapshot");
    }
}
