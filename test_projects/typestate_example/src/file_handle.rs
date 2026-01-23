use std::marker::PhantomData;
use std::path::PathBuf;

/// State: File is closed
pub struct Closed;

/// State: File is open
pub struct Open;

/// A file handle that uses typestate to ensure files are opened before reading.
///
/// INVARIANT: Files must be in the Open state before they can be read or written to.
/// This is enforced at compile time using PhantomData.
pub struct FileHandle<S> {
    path: PathBuf,
    _state: PhantomData<S>,
}

impl FileHandle<Closed> {
    /// Create a new file handle in the Closed state
    pub fn new(path: PathBuf) -> Self {
        FileHandle {
            path,
            _state: PhantomData,
        }
    }

    /// Open the file, transitioning to the Open state
    ///
    /// This consumes self and returns a FileHandle<Open>,
    /// ensuring the file can only be opened once.
    pub fn open(self) -> FileHandle<Open> {
        println!("Opening file: {:?}", self.path);
        FileHandle {
            path: self.path,
            _state: PhantomData,
        }
    }
}

impl FileHandle<Open> {
    /// Read from the file (only available in Open state)
    ///
    /// INVARIANT: Can only be called on an open file handle
    pub fn read(&self) -> Vec<u8> {
        println!("Reading from file: {:?}", self.path);
        vec![1, 2, 3, 4]
    }

    /// Write to the file (only available in Open state)
    ///
    /// INVARIANT: Can only be called on an open file handle
    pub fn write(&mut self, data: &[u8]) {
        println!("Writing {} bytes to file: {:?}", data.len(), self.path);
    }

    /// Close the file, transitioning back to Closed state
    pub fn close(self) -> FileHandle<Closed> {
        println!("Closing file: {:?}", self.path);
        FileHandle {
            path: self.path,
            _state: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typestate_workflow() {
        let file = FileHandle::new(PathBuf::from("test.txt"));
        let mut file = file.open();
        file.write(b"hello");
        let _data = file.read();
        let _closed = file.close();
    }
}
