# File Watcher

### Usage

First, add the following to your `Cargo.toml`

    [dependencies]
    filewatcher = "0.1.0"

Example

    extern crate filewatcher;
    use filewatcher::{FileWatcher, Message};
	
	fn main() {
		let mut times = 0;
		let mut watcher = match FileWatcher::new("Cargo.toml".to_string()) {
			Ok(w) => w,
			Err(err) => panic!("Can't read: {}", err)
		};
	
		let inode = watcher.get_inode();
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

