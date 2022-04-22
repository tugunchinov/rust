#![forbid(unsafe_code)]

use std::path::Path;

pub enum Handle<'a> {
    Dir(DirHandle<'a>),
    File(FileHandle<'a>),
    Content {
        file_path: &'a Path,
        content: &'a [u8],
    },
}

pub struct DirHandle<'a> {
    pub path: &'a Path,
    pub descend: bool,
}

impl<'a> DirHandle<'a> {
    pub fn descend(&mut self) {
        self.descend = true
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}

pub struct FileHandle<'a> {
    pub path: &'a Path,
    pub read: bool,
}

impl<'a> FileHandle<'a> {
    pub fn read(&mut self) {
        self.read = true
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}
