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
    version: usize,

}

unsafe impl Sync for Manager {}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            version: 0,
        }
    }

    /// Returns the 'version' of the manager. The version is
    /// increase everytime a pack is added or removed.
    pub fn version(&self) -> usize {
        self.version
    }

    pub fn open(&self, _plugin: &str, _name: &str) -> Option<Box<io::Read>> {
        None
    }

    pub fn open_all(&self, _plugin: &str, _name: &str) -> Vec<Box<io::Read>> {
        Vec::new()
    }
}

