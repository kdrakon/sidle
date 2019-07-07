use std::fs::DirEntry;
use std::path::PathBuf;

use crate::error_code;
use crate::error_code::ErrorCode;

#[derive(Clone)]
pub enum DirObject {
    Dir { name: String, path: PathBuf },
    File { name: String, path: PathBuf },
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
            Err(error_code::UNKNOWN_DIR_OBJECT_FOUND)
        }
    }
}
