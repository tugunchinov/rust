#![forbid(unsafe_code)]
use std::{fs, io, path::Path};

use crate::handle::{DirHandle, FileHandle, Handle};

type Callback<'a> = dyn FnMut(&mut Handle) + 'a;

#[derive(Default)]
pub struct Walker<'a> {
    callbacks: Vec<Box<Callback<'a>>>,
}

impl<'a> Walker<'a> {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    pub fn add_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&mut Handle) + 'a,
    {
        self.callbacks.push(Box::new(callback));
    }

    pub fn walk<P: AsRef<Path>>(mut self, path: P) -> io::Result<()> {
        if self.callbacks.is_empty() {
            return Ok(());
        }

        let mut callbacks = Vec::new();

        for callback in self.callbacks.iter_mut() {
            callbacks.push(&mut **callback);
        }

        Self::traverse(path, &mut callbacks)
    }

    fn partition(handle: &mut Handle, callbacks: &mut [&mut Callback]) -> usize {
        if callbacks.is_empty() {
            return 0;
        }

        let (mut i, mut j) = (0usize, 1usize);

        while j < callbacks.len() {
            callbacks[j](handle);

            if let Handle::File(handle) = handle {
                if handle.read {
                    if i != j {
                        callbacks.swap(i, j);
                        j -= 1;
                    }

                    i += 1;
                }
                handle.read = false;
            } else if let Handle::Dir(handle) = handle {
                if handle.descend {
                    if i != j {
                        callbacks.swap(i, j);
                        j -= 1;
                    }

                    i += 1;
                }
                handle.descend = false;
            } else {
                panic!();
            };

            j += 1;
        }

        if i == 0 {
            callbacks[0](handle);
            if let Handle::File(handle) = handle {
                if handle.read {
                    i += 1;
                }
                handle.read = false;
            } else if let Handle::Dir(handle) = handle {
                if handle.descend {
                    i += 1;
                }
                handle.descend = false;
            } else {
                panic!();
            };
        }

        i
    }

    fn traverse<P: AsRef<Path>>(path: P, callbacks: &mut [&mut Callback]) -> io::Result<()> {
        let mut result = Ok(());

        if path.as_ref().is_dir() {
            for entry in (path.as_ref().read_dir()?).flatten() {
                let path_buf = entry.path();

                let i = if entry.path().is_dir() {
                    Self::partition(
                        &mut Handle::Dir(DirHandle {
                            path: path_buf.as_path(),
                            descend: false,
                        }),
                        callbacks,
                    )
                } else {
                    Self::partition(
                        &mut Handle::File(FileHandle {
                            path: path_buf.as_path(),
                            read: false,
                        }),
                        callbacks,
                    )
                };

                if i > 0 {
                    result = Self::traverse(entry.path(), &mut callbacks[0..i]);
                }
            }
        } else if path.as_ref().is_file() {
            let buf = fs::read(&path).expect("failed");
            let mut content = Handle::Content {
                file_path: path.as_ref(),
                content: &buf,
            };

            for callback in callbacks {
                callback(&mut content);
            }
        } else if !path.as_ref().exists() {
            return Err(std::io::Error::from_raw_os_error(1));
        }

        result
    }
}
