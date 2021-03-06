use std::fs::File;
use std::io::SeekFrom;
use std::io::BufReader;
use std::io::prelude::*;
use std::os::unix::fs::MetadataExt;
use std::io::ErrorKind;

pub struct FileWatcher {
    filename: String,
    inode: u64,
    position: u64,
    reader: BufReader<File>,
	finish: bool
}

impl Clone for FileWatcher {
    fn clone(&self) -> FileWatcher {
        let file = File::open(&self.filename).unwrap();
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(self.position)).unwrap();

        FileWatcher { 
            filename: self.filename.clone(),
            inode: self.inode,
            position: self.position,
            reader: reader,
            finish: self.finish,
        }
    }
}

pub enum Message {
	NONE,
	Line { inode: u64, position: u64, line: String }
}

impl FileWatcher {
    pub fn new(filename: String) -> Result<FileWatcher, ::std::io::Error> {
        let file = match File::open(filename.clone()) {
            Ok(f) => f,
            Err(err) => return Err(err)
        };
        
        let metadata = match file.metadata() {
            Ok(m) => m,
            Err(err) => return Err(err)
        };

        let mut reader = BufReader::new(file);
		let position = metadata.len();
        reader.seek(SeekFrom::Start(position)).unwrap();
        Ok(FileWatcher{filename: filename,
                      inode: metadata.ino(),
                      position: position,
                      reader: reader,
                      finish: false})
    }
	
	
	pub fn reposition(&mut self, inode: u64, start_pos: u64) -> Result<FileWatcher, &'static str> {
		if inode > 0 && self.inode != inode {
			return Err("last watcher file inode is can't be match!");
		}
		self.position = start_pos;
        self.reader.seek(SeekFrom::Start(self.position)).unwrap();
		Ok(self.clone())
	}
	
	pub fn get_filename(&mut self) -> String {
		self.filename.clone()
	}
	
	pub fn get_inode(&mut self) -> u64 {
		self.inode
	}
	
	pub fn get_position(&mut self) -> u64 {
		self.position
	}
	
	pub fn close(&mut self){
		self.finish = true;
	}

    fn reopen(&mut self){
        loop {
            match File::open(self.filename.clone()) {
                Ok(f) => {
                    let metadata = match f.metadata() {
                        Ok(m) => m,
                        Err(_) => {
                            continue;
                        }
                    };
					self.reader = BufReader::new(f);
                    if metadata.ino() != self.inode{
                        self.position = 0;
                        self.inode = metadata.ino();
                    }
					self.reader.seek(SeekFrom::Start(self.position)).unwrap();
                    break;
                },
                Err(err) => {
                    if err.kind() == ErrorKind::NotFound{
						if self.finish {
							break;
						}
                        continue;
                    }
                }
            };
        }
    }

    fn read(&mut self) -> Option<Message> {
        let mut line = String::new();
        let resp = self.reader.read_line(&mut line);
        match resp {
			Ok(0) => {
                if self.finish {
                    None
                } else {
                    self.reopen();
					Some(Message::NONE)
                }
			},
            Ok(len) => {
                if self.finish {
                    return None;
                }
                self.position += len as u64;
                self.reader.seek(SeekFrom::Start(self.position)).unwrap();
				Some(Message::Line{ inode: self.inode, position: self.position, line: line })
            },
            Err(err) => panic!("Can't read: {}", err)
        }
    }
}


impl Iterator for FileWatcher {
    type Item = Message;

    fn next(&mut self) -> Option<Message> {
		self.read()
    }
}


#[cfg(test)]
mod tests {
	use super::{FileWatcher, Message};

    #[test]
    fn it_works() {
		let mut times = 0;
		let mut watcher = match FileWatcher::new("Cargo.toml".to_string()) {
			Ok(w) => w,
			Err(err) => panic!("Can't read: {}", err)
		};
		
		let inode = watcher.inode;
		let mut watcher = match watcher.reposition(inode, 0) {
				Ok(w) => w,
				Err(err) => panic!("Can't reposition: {}", err)
			};

		loop {
		    match watcher.next() {
				Some(Message::NONE) => {
					println!("None None!!!");
				},
		        Some(Message::Line{inode, position, line}) => {
					println!("inode: {:?}  position: {:?} line: {:?}", inode, position, line);	
		        },
		        None => break
		    }
			
			println!("filename: {:?}", watcher.get_filename());
			println!("file inode: {:?}", watcher.get_inode());
			println!("file position: {:?}", watcher.get_position());
			
			if times == 5 {
				watcher.close();
			}
			times += 1;
		}
    }
}