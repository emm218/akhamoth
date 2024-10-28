use std::{fs, io, path::PathBuf, rc::Rc};

pub struct SourceFile {
    /// path to this file on disk
    pub path: PathBuf,
    /// the full source of the file
    pub src: String,
    /// absolute start position of this file in source map
    pub start_pos: u32,
    /// relative byte offsets of each newline
    lines: Vec<u32>,
}

impl SourceFile {
    pub fn new<P: Into<PathBuf>>(path: P, start_pos: u32) -> Result<Self, io::Error> {
        let path = path.into();
        let src = fs::read_to_string(&path)?;

        let lines = src
            .bytes()
            .enumerate()
            .filter(|&(_, b)| b == b'\n')
            .map(|(i, _)| i as u32)
            .collect::<Vec<_>>();

        Ok(Self {
            path,
            src,
            lines,
            start_pos,
        })
    }

    pub fn line_number(&self, byte_offset: u32) -> usize {
        self.lines.partition_point(|&x| x < byte_offset)
    }

    pub fn end_position(&self) -> u32 {
        self.start_pos + self.src.len() as u32
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("4GB limit for source files reached")]
    OffsetOverflowError,
}

#[derive(Default)]
pub struct SourceMap {
    files: Vec<Rc<SourceFile>>,
}

impl SourceMap {
    pub fn load_file<P: Into<PathBuf>>(&mut self, path: P) -> Result<Rc<SourceFile>, LoadError> {
        let start_pos = match self.files.last() {
            Some(f) => f
                .end_position()
                .checked_add(1)
                .ok_or(LoadError::OffsetOverflowError)?,
            None => 0,
        };

        let source_file = Rc::new(SourceFile::new(path.into(), start_pos)?);

        self.files.push(source_file.clone());

        Ok(source_file)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    lo: u32,
    len: u32,
}

impl Span {
    pub fn new(lo: u32, len: u32) -> Self {
        Self { lo, len }
    }
}
