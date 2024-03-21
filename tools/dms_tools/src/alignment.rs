use std::{fs, collections::HashMap};

#[derive(Debug, Clone)]
pub struct Record {
    pub query: SequenceRef,
    pub reference: SequenceRef,
    pub strand_match: bool,
    pub num_matching_bases: usize,
    pub num_mapped_bases: usize,
    pub mapping_quality: u8,
    pub fields: HashMap<String, String>,
    pub alignment: Alignment
}

pub struct RecordReader {
    raw_records_iter: csv::StringRecordsIntoIter<fs::File>,
}

impl RecordReader {
    #[allow(non_snake_case)]
    pub fn read(PAF_file: &std::path::Path) -> Result<Self, Error> {
        let records_reader = {
            let mut reader_builder = csv::ReaderBuilder::new();
            reader_builder.delimiter(b'\t');
            reader_builder.has_headers(false);
            reader_builder.double_quote(false);
            reader_builder.escape(None);
            reader_builder.quoting(false);
            reader_builder.flexible(true);
            reader_builder.from_path(PAF_file)?
        };

        Ok(Self {
            raw_records_iter: records_reader.into_records()
        })
    }

    fn parse_single_record(raw_record: csv::StringRecord) -> Result<Record, Error> {
        lazy_static! {
            static ref ALIGNMENT_MATCHER: regex::Regex = regex::Regex::new(r"(=[ACTGN]+|\*[actgn][actgn]|\+[actgn]+|\-[actgn]+)").expect("failed to compile PAF alignment regex");
        }

        let query = SequenceRef {
            name: raw_record.get(0).ok_or(Error::MissingField("query_name".to_string()))?.to_string(),
            length: raw_record.get(1).ok_or(Error::MissingField("query_length".to_string()))?.parse::<usize>()?,
            start: raw_record.get(2).ok_or(Error::MissingField("query_start".to_string()))?.parse::<usize>()?,
            end: raw_record.get(3).ok_or(Error::MissingField("query_end".to_string()))?.parse::<usize>()?
        };
        let strand_match = raw_record.get(4).ok_or(Error::MissingField("strand".to_string()))? == "+";
        let reference = SequenceRef {
            name: raw_record.get(5).ok_or(Error::MissingField("target_name".to_string()))?.to_string(),
            length: raw_record.get(6).ok_or(Error::MissingField("target_length".to_string()))?.parse::<usize>()?,
            start: raw_record.get(7).ok_or(Error::MissingField("target_start".to_string()))?.parse::<usize>()?,
            end: raw_record.get(8).ok_or(Error::MissingField("target_end".to_string()))?.parse::<usize>()?
        };
        let num_matching_bases = raw_record.get(9).ok_or(Error::MissingField("num_matches".to_string()))?.parse::<usize>()?;
        let num_mapped_bases = raw_record.get(10).ok_or(Error::MissingField("alignment_length".to_string()))?.parse::<usize>()?;
        let mapping_quality = raw_record.get(11).ok_or(Error::MissingField("mapping_quality".to_string()))?.parse::<u8>()?;

        let mut fields = HashMap::new();
        for raw_field in raw_record.iter().skip(12) {
            let raw_tokens: Vec<&str> = raw_field.splitn(3, ":").collect();
            fields.insert(raw_tokens[0].to_string(), raw_tokens[2].to_string());
        }

        let alignment: Alignment = fields.get("cs").map_or(vec![], |raw_alignment| {
            ALIGNMENT_MATCHER.captures_iter(raw_alignment)
                .filter_map(|capture| capture.get(0))
                .map(|match_| match_.as_str())
                .map(|raw| {
                    let mut characters = raw.chars();
                    match characters.next().expect("invalid cs tag") {
                        '=' => { AlignmentOperation::Identical(raw.trim_start_matches("=").to_string()) },
                        '*' => { AlignmentOperation::Substitution(characters.next().unwrap(), characters.next().unwrap())},
                        '+' => { AlignmentOperation::Insertion(raw.trim_start_matches("+").to_string()) },
                        '-' => { AlignmentOperation::Deletion(raw.trim_start_matches("-").to_string()) },
                        _ => { panic!("invalid cs tag"); }
                    }
                })
            .collect()
        }).into();

        Ok(Record {
            query: query,
            reference: reference,
            strand_match: strand_match,
            num_matching_bases: num_matching_bases,
            num_mapped_bases: num_mapped_bases,
            mapping_quality: mapping_quality,
            fields: fields,
            alignment: alignment
        })
    }
}

impl Iterator for RecordReader {
    type Item = Result<Record, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.raw_records_iter.next() {
            Some(Ok(raw_record)) => {
                match Self::parse_single_record(raw_record) {
                    Ok(record) => Some(Ok(record)),
                    Err(error) => Some(Err(error.into()))
                }
            },
            Some(Err(error)) => {
                Some(Err(error.into()))
            },
            None => None
        }
    }
}

impl Record {
    pub fn alignment_subset(&self, reference_start: usize, reference_end: usize) -> Option<Alignment> {
        #[derive(Debug)]
        enum State {
            Before,
            In,
        }

        if self.reference.start > reference_start || self.reference.end < reference_end {
            return None
        }

        let mut state = State::Before;
        let mut position_in_reference = self.reference.start;
        let mut subset: Vec<AlignmentOperation> = Vec::new();

        for alignment_operation in self.alignment.operations().iter() {
            let operation_length_relative_to_reference = alignment_operation.length_relative_to_reference();
            position_in_reference += operation_length_relative_to_reference;
            match state {
                State::Before => {
                    if position_in_reference >= reference_start {
                        if position_in_reference >= reference_end {
                            let start_offset = alignment_operation.raw_length().saturating_sub(position_in_reference - reference_start);
                            let stop_offset = alignment_operation.raw_length().saturating_sub(position_in_reference - reference_end);
                            subset.push(alignment_operation.range_from_to(start_offset, stop_offset));
                            break;
                        } else {
                            let offset = alignment_operation.raw_length().saturating_sub(position_in_reference - reference_start);
                            subset.push(alignment_operation.range_from(offset));
                            state = State::In;
                        }
                    }
                },
                State::In => {
                    if position_in_reference < reference_end {
                        subset.push(alignment_operation.clone());
                    } else {
                        let offset = alignment_operation.raw_length().saturating_sub(position_in_reference - reference_end);
                        subset.push(alignment_operation.range_to(offset));
                        break;
                    }
                },
            }
        }

        Some(subset.into())
    }
}

#[derive(Clone, Debug)]
pub struct SequenceRef {
    pub name: String,
    pub length: usize,
    pub start: usize,
    pub end: usize
}

#[derive(Debug, Clone)]
pub struct Alignment {
    operations: Vec<AlignmentOperation>
}

impl Alignment {
    pub fn operations(&self) -> &[AlignmentOperation] {
        &self.operations
    }

    pub fn raw_length(&self) -> usize {
        self.operations().into_iter().map(|operation| {
            operation.raw_length()
        }).sum()
    }

    pub fn length_relative_to_reference(&self) -> usize {
        self.operations().into_iter().map(|operation| {
            operation.length_relative_to_reference()
        }).sum()
    }

    pub fn make_sequences(&self) -> (String, String) {
        let mut merged_reference_sequence = String::new();
        let mut merged_query_sequence = String::new();
        for operation in self.operations().iter() {
            match operation {
                AlignmentOperation::Identical(sequence) => {
                    merged_reference_sequence.push_str(sequence);
                    merged_query_sequence.push_str(sequence);
                },
                AlignmentOperation::Substitution(reference, query) => {
                    merged_reference_sequence.push_str(&reference.to_string());
                    merged_query_sequence.push_str(&query.to_string());
                },
                AlignmentOperation::Insertion(sequence) => {
                    merged_query_sequence.push_str(sequence);
                },
                AlignmentOperation::Deletion(sequence) => {
                    merged_reference_sequence.push_str(sequence);
                },
            };
        }
        (merged_reference_sequence, merged_query_sequence)
    }

    pub fn call_coding_variants(&self) -> Result<Vec<(char, usize, char)>, Error> {
        lazy_static! {
            static ref TRANSLATION_TABLE: HashMap<&'static str, char> = [
                ("ATA", 'I'),
                ("ATC", 'I'),
                ("ATT", 'I'),
                ("ATG", 'M'),
                ("ACA", 'T'),
                ("ACC", 'T'),
                ("ACG", 'T'),
                ("ACT", 'T'),
                ("AAC", 'N'),
                ("AAT", 'N'),
                ("AAA", 'K'),
                ("AAG", 'K'),
                ("AGC", 'S'),
                ("AGT", 'S'),
                ("AGA", 'R'),
                ("AGG", 'R'),                
                ("CTA", 'L'),
                ("CTC", 'L'),
                ("CTG", 'L'),
                ("CTT", 'L'),
                ("CCA", 'P'),
                ("CCC", 'P'),
                ("CCG", 'P'),
                ("CCT", 'P'),
                ("CAC", 'H'),
                ("CAT", 'H'),
                ("CAA", 'Q'),
                ("CAG", 'Q'),
                ("CGA", 'R'),
                ("CGC", 'R'),
                ("CGG", 'R'),
                ("CGT", 'R'),
                ("GTA", 'V'),
                ("GTC", 'V'),
                ("GTG", 'V'),
                ("GTT", 'V'),
                ("GCA", 'A'),
                ("GCC", 'A'),
                ("GCG", 'A'),
                ("GCT", 'A'),
                ("GAC", 'D'),
                ("GAT", 'D'),
                ("GAA", 'E'),
                ("GAG", 'E'),
                ("GGA", 'G'),
                ("GGC", 'G'),
                ("GGG", 'G'),
                ("GGT", 'G'),
                ("TCA", 'S'),
                ("TCC", 'S'),
                ("TCG", 'S'),
                ("TCT", 'S'),
                ("TTC", 'F'),
                ("TTT", 'F'),
                ("TTA", 'L'),
                ("TTG", 'L'),
                ("TAC", 'Y'),
                ("TAT", 'Y'),
                ("TAA", 'X'),
                ("TAG", 'X'),
                ("TGC", 'C'),
                ("TGT", 'C'),
                ("TGA", 'X'),
                ("TGG", 'W'),
            ].into_iter().collect();
        }

        let (reference, query) = self.make_sequences();

        if reference.len()%3 != 0 {
            return Err(Error::InvalidSequenceOperation(format!("cannot call coding variants in sequence region with length that is not a multiple of three (length = {})", reference.len())));
        } else if reference.len() != query.len() {
            return Err(Error::InvalidSequenceOperation(format!("cannot call coding variants for a region with indels (reference length = {}, query length = {})", reference.len(), query.len())));
        } else if query.len() <= 0 {
            return Err(Error::InvalidSequenceOperation(format!("cannot call coding variants for a zero-length region")));
        }

        let mut variants = Vec::new();
        for i in (0..reference.len()).step_by(3) {
            let reference_aa = TRANSLATION_TABLE.get(reference[i..i+3].to_uppercase().as_str()).unwrap_or(&'?');
            let query_aa = TRANSLATION_TABLE.get(query[i..i+3].to_uppercase().as_str()).unwrap_or(&'?');
            if reference_aa != query_aa {
                variants.push((*reference_aa, i/3 + 1, *query_aa))
            }
        }
        Ok(variants)
    }

    pub fn call_variants(&self) -> Vec<String> {
        let mut variants: Vec<String> = Vec::new();
        let mut position = 0;
        for operation in self.operations.iter() {
            match operation {
                AlignmentOperation::Identical(_) => {

                },
                AlignmentOperation::Substitution(reference, query) => {
                    variants.push(format!("{}{}{}", reference, position, query))
                    // Maybe this should collapse consecutive substitutions into a single "variant"?
                }, 
                AlignmentOperation::Insertion(sequence) => {
                    variants.push(format!("{}+{}", position, sequence))
                },
                AlignmentOperation::Deletion(sequence) => {
                    variants.push(format!("{}-{}", position, sequence))
                },
            };
            position += operation.length_relative_to_reference();
        }
        variants
    }
}

impl From<Vec<AlignmentOperation>> for Alignment {
    fn from(source: Vec<AlignmentOperation>) -> Self {
        Self {
            operations: source
        }
    }
}

#[derive(Clone, Debug)]
pub enum AlignmentOperation {
    Identical(String),
    Substitution(char, char),
    Insertion(String),
    Deletion(String)
}

impl AlignmentOperation {
    pub fn length_relative_to_reference(&self) -> usize {
        match self {
            Self::Identical(sequence) => sequence.len(),
            Self::Substitution(_, _) => 1,
            Self::Insertion(_) => 0,
            Self::Deletion(sequence) => sequence.len(),
        }
    }

    pub fn raw_length(&self) -> usize {
        match self {
            Self::Identical(sequence) => sequence.len(),
            Self::Substitution(_, _) => 1,
            Self::Insertion(sequence) => sequence.len(),
            Self::Deletion(sequence) => sequence.len(),
        }
    }

    pub fn range_to(&self, stop: usize) -> Self {
        match self {
            Self::Identical(sequence) => Self::Identical(sequence[..stop].into()),
            Self::Substitution(reference, query) => Self::Substitution(*reference, *query),
            Self::Insertion(sequence) => Self::Insertion(sequence[..stop].into()),
            Self::Deletion(sequence) => Self::Deletion(sequence[..stop].into()),
        }
    }

    pub fn range_from(&self, start: usize) -> Self {
        match self {
            Self::Identical(sequence) => Self::Identical(sequence[start..].into()),
            Self::Substitution(reference, query) => Self::Substitution(*reference, *query),
            Self::Insertion(sequence) => Self::Insertion(sequence[start..].into()),
            Self::Deletion(sequence) => Self::Deletion(sequence[start..].into()),
        }
    }

    pub fn range_from_to(&self, start: usize, stop: usize) -> Self {
        match self {
            Self::Identical(sequence) => Self::Identical(sequence[start..stop].into()),
            Self::Substitution(reference, query) => Self::Substitution(*reference, *query),
            Self::Insertion(sequence) => Self::Insertion(sequence[start..stop].into()),
            Self::Deletion(sequence) => Self::Deletion(sequence[start..stop].into()),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Reading(csv::Error),
    Parsing,
    MissingField(String),
    InvalidSequenceOperation(String)
}

impl std::error::Error for Error {

}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reading(error) => write!(f, "input error: {}", error),
            Self::Parsing => write!(f, "failed to parse input"),
            Self::MissingField(name) => write!(f, "expected field \"{}\"", name),
            Self::InvalidSequenceOperation(detail) => write!(f, "invalid sequence: {}", detail)
        }
    }
}

impl From<csv::Error> for Error {
    fn from(source: csv::Error) -> Self {
        Self::Reading(source)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_: std::num::ParseIntError) -> Self {
        Self::Parsing
    }
}
