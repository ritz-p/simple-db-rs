pub mod block_id;
pub mod page;

use std::collections::HashMap;
use std::fs::{create_dir_all, read_dir, remove_file, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use block_id::BlockId;
use page::Page;

#[derive(Debug)]
pub struct FileManager {
    db_directory: PathBuf,
    block_size: u64,
    pub is_new: bool,
    open_files: HashMap<String, File>,
}

impl FileManager {
    pub fn new(db_directory: PathBuf, block_size: u64) -> Self {
        let is_new = !db_directory.exists();

        if is_new {
            create_dir_all(&db_directory).expect("Failed to create db directory");
        }

        if let Ok(entries) = read_dir(&db_directory) {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .map_or(false, |filename| filename.starts_with("temp"))
                })
                .for_each(|path| {
                    let _ = remove_file(path);
                });
        }

        FileManager {
            db_directory,
            block_size,
            is_new,
            open_files: HashMap::new(),
        }
    }

    pub fn read(&mut self, block_id: &BlockId, page: &mut Page) {
        let block_size = self.block_size;
        let file = self.get_file(&block_id.filename);
        let pos = (block_id.number as u64) * block_size;
        if let Err(e) = file.seek(SeekFrom::Start(pos)) {
            panic!("cannot read block {:?} because {:?}", block_id, e);
        }

        if let Err(e) = file.read_exact(page.contents_mut()) {
            panic!("cannot read block {:?} because {:?}", block_id, e);
        }
    }

    pub fn write(&mut self, block_id: BlockId, page: &Page) {
        let block_size = self.block_size;
        let file = self.get_file(&block_id.filename);
        let pos = (block_id.number as u64) * block_size;
        if let Err(e) = file.seek(SeekFrom::Start(pos)) {
            panic!("cannot write block {:?} because {:?}", block_id, e);
        }

        if let Err(e) = file.write_all(page.contents()) {
            panic!("cannot write block {:?} because {:?}", block_id, e);
        }
    }

    pub fn append(&mut self, filename: &str) -> BlockId {
        let block_size = self.block_size;
        let new_block_number = self.length(filename) as i32;

        let block_id = BlockId {
            filename: filename.to_string(),
            number: new_block_number,
        };

        let file = self.get_file(&block_id.filename);
        let empty_bytes = vec![0u8; block_size as usize];

        let pos = (block_id.number as u64) * block_size;
        if let Err(e) = file.seek(SeekFrom::Start(pos)) {
            panic!("cannot append block {:?} because {:?}", block_id, e);
        }

        if let Err(e) = file.write_all(&empty_bytes) {
            panic!("cannot append block {:?} because {:?}", block_id, e);
        }
        block_id
    }

    pub fn length(&mut self, filename: &str) -> u64 {
        let file = self.get_file(filename);
        match file.metadata() {
            Ok(metadata) => metadata.len() / self.block_size,
            Err(e) => {
                panic!("cannot access filename {:?} because {:?}", filename, e)
            }
        }
    }

    fn get_file(&mut self, filename: &str) -> &mut File {
        self.open_files
            .entry(filename.to_string())
            .or_insert_with(|| {
                let path = self.db_directory.join(filename);
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)
                    .unwrap_or_else(|_| panic!("cannot open file: {:?}", path))
            })
    }
}
