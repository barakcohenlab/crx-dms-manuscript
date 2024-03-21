use std::{fmt, io::{self, prelude::*}};

#[derive(Debug)]
pub struct FASTQRecord {
    pub identifier: String,
    pub sequence: String,
    pub quality_scores: Vec<u8>
}

impl fmt::Display for FASTQRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}\n{}\n+\n{}\n", self.identifier, self.sequence, unsafe { std::str::from_utf8_unchecked(&self.quality_scores) })
    }
}

pub struct FASTQReader<R: Read> {
    source: io::BufReader<R>,
    buffer: String
}

impl <R: Read> FASTQReader<R> {
    pub fn read_fastq(source: R) -> FASTQReader<R> {
        FASTQReader {
            source: io::BufReader::new(source),
            buffer: String::new()
        }
    }
}

impl <R: Read> Iterator for FASTQReader<R> {
    type Item = Result<FASTQRecord, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {

        let identifier: String = match self.source.read_line(&mut self.buffer) {
            Err(error) => { return Some(Err(error)); },
            Ok(0) => { return None; },
            Ok(_) => { self.buffer.trim_start_matches('@').trim_end().to_owned() },
        };
        self.buffer.clear();

        let sequence: String = match self.source.read_line(&mut self.buffer) {
            Err(error) => { return Some(Err(error)); },
            Ok(0) => { return Some(Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "file ended in the middle of record"))) },
            Ok(_) => { self.buffer.trim_end().to_owned() },
        };
        self.buffer.clear();

        match self.source.read_line(&mut self.buffer) {
            Err(error) => { return Some(Err(error)); },
            Ok(0) => { return Some(Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "file ended in the middle of record"))) },
            Ok(_) => { },
        };
        self.buffer.clear();

        let quality_scores: Vec<u8> = match self.source.read_line(&mut self.buffer) {
            Err(error) => { return Some(Err(error)); },
            Ok(0) => { return Some(Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "file ended in the middle of record"))) },
            Ok(_) => { self.buffer.trim_end().into() },
        };
        self.buffer.clear();

        Some(Ok(FASTQRecord {
            identifier: identifier,
            sequence: sequence,
            quality_scores: quality_scores
        }))
    }
}

