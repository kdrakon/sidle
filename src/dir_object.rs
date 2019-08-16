use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use crate::error_code;
use crate::error_code::ErrorCode;
use std::cmp::Ordering;

#[derive(Clone)]
pub enum DirObject {
    Dir { name: String, path: PathBuf },
    File { name: String, path: PathBuf },
    Unknown { name: String, path: PathBuf },
}

trait HasFileName {
    fn filename(&self) -> &str;
}

impl HasFileName for DirObject {
    fn filename(&self) -> &str {
        match self {
            DirObject::Dir { name, .. } => name,
            DirObject::File { name, .. } => name,
            DirObject::Unknown { name, .. } => name,
        }
    }
}

pub trait IntoDirObject {
    fn new_dir_object(&self) -> Result<DirObject, ErrorCode>;
}

impl IntoDirObject for DirEntry {
    fn new_dir_object(&self) -> Result<DirObject, ErrorCode> {
        let metadata = self.metadata().map_err(|_| error_code::COULD_NOT_READ_METADATA)?;
        let name = self.file_name().into_string().map_err(|_| error_code::COULD_NOT_READ_METADATA)?;
        let path = self.path();
        if metadata.is_dir() {
            Ok(DirObject::Dir { name, path })
        } else if metadata.is_file() {
            Ok(DirObject::File { name, path })
        } else {
            Ok(DirObject::Unknown { name, path })
        }
    }
}

pub fn dir_ordering(a: &DirObject, b: &DirObject) -> Ordering {
    match (a, b) {
        (DirObject::Dir { name: a_name, .. }, DirObject::Dir { name: b_name, .. }) => name_ordering(a_name, b_name),
        (DirObject::Dir { .. }, _) => Ordering::Less,
        (_, DirObject::Dir { .. }) => Ordering::Greater,
        (_, _) => name_ordering(a.filename(), b.filename()),
    }
}

fn name_ordering(a: &str, b: &str) -> Ordering {
    match (a, b) {
        (a, b) if a.starts_with('.') ^ b.starts_with('.') => {
            if a.starts_with('.') {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        (a, b) => a.cmp(b),
    }
}
