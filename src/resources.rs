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

extern crate steven_resources as internal;

use std::thread;
use std::path;
use std::io;
use std::fs;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

pub trait Pack: Sync + Send {
    fn open(&self, name: &str) -> Option<Box<io::Read>>;
}

pub struct Manager {
    packs: Vec<Box<Pack>>,
    version: usize,

    vanilla_chan: Option<mpsc::Receiver<bool>>,
    vanilla_assets_chan: Option<mpsc::Receiver<bool>>,
    vanilla_progress: Arc<Mutex<Progress>>,
}

pub struct ManagerUI {
    progress_ui: Vec<ProgressUI>,
    num_tasks: isize,
}

struct ProgressUI {
    task_name: String,
    task_file: String,
    position: f64,
    closing: bool,
    progress: f64,
}

struct Progress {
    tasks: Vec<Task>,
}

struct Task {
    task_name: String,
    task_file: String,
    total: u64,
    progress: u64,
}

unsafe impl Sync for Manager {}

impl Manager {
    pub fn new() -> (Manager, ManagerUI) {
        let mut m = Manager {
            packs: Vec::new(),
            version: 0,
            vanilla_chan: None,
            vanilla_assets_chan: None,
            vanilla_progress: Arc::new(Mutex::new(Progress {
                tasks: vec![],
            })),
        };
        (m, ManagerUI { progress_ui: vec!{}, num_tasks: 0 })
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

    fn add_pack(&mut self, pck: Box<Pack>) {
    }

    fn load_vanilla(&mut self) {
    }

    fn load_assets(&mut self) {
    }

    fn download_assets(&mut self) {
    }

    fn download_vanilla(&mut self) {
    }

    fn add_task(progress: &Arc<Mutex<Progress>>, name: &str, file: &str, length: u64) {
    }

    fn add_task_progress(progress: &Arc<Mutex<Progress>>, name: &str, file: &str, prog: u64) {
    }
}

struct DirPack {
    root: path::PathBuf,
}

impl Pack for DirPack {
    fn open(&self, name: &str) -> Option<Box<io::Read>> {
        match fs::File::open(self.root.join(name)) {
            Ok(val) => Some(Box::new(val)),
            Err(_) => None,
        }
    }
}

struct InternalPack;

impl Pack for InternalPack {
    fn open(&self, name: &str) -> Option<Box<io::Read>> {
        match internal::get_file(name) {
            Some(val) => Some(Box::new(io::Cursor::new(val))),
            None => None,
        }
    }
}

struct ProgressRead<'a, T> {
    read: T,
    progress: &'a Arc<Mutex<Progress>>,
    task_name: String,
    task_file: String,
}

impl <'a, T: io::Read> io::Read for ProgressRead<'a, T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = self.read.read(buf)?;
        Manager::add_task_progress(self.progress, &self.task_name, &self.task_file, size as u64);
        Ok(size)
    }
}
