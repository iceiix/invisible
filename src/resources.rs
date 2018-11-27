// Copyright 2016 Matthew Collins
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io;

pub trait Pack: Sync + Send {
    fn open(&self, name: &str) -> Option<Box<io::Read>>;
}

pub struct Manager {
    packs: Vec<Box<Pack>>,
    version: usize,

}

unsafe impl Sync for Manager {}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            packs: Vec::new(),
            version: 0,
        }
    }

    /// Returns the 'version' of the manager. The version is
    /// increase everytime a pack is added or removed.
    pub fn version(&self) -> usize {
        self.version
    }

    pub fn open(&self, plugin: &str, name: &str) -> Option<Box<io::Read>> {
        let path = format!("assets/{}/{}", plugin, name);
        for pack in self.packs.iter().rev() {
            if let Some(val) = pack.open(&path) {
                return Some(val);
            }
        }
        None
    }

    pub fn open_all(&self, plugin: &str, name: &str) -> Vec<Box<io::Read>> {
        let mut ret = Vec::new();
        let path = format!("assets/{}/{}", plugin, name);
        for pack in self.packs.iter().rev() {
            if let Some(val) = pack.open(&path) {
                ret.push(val);
            }
        }
        ret
    }
}

