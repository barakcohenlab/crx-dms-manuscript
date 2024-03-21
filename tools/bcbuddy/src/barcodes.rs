pub struct BarcodeExtractor {
    matcher: regex::Regex,
    capture_group_names: Vec<String>
}


impl BarcodeExtractor {
    pub fn new(raw_matcher: &str) -> Result<Self, BarcodeError> {
        let matcher = regex::Regex::new(raw_matcher)?;
        let capture_group_names: Vec<String> = matcher.capture_names().filter_map(|maybe_name| maybe_name.map(|name| name.to_string())).collect();
        if matcher.captures_len() == 0{
            Err(BarcodeError::RegexMissingCaptures(raw_matcher.to_string()))
        } else {
            Ok(Self {
                matcher,
                capture_group_names,
            })
        }
    }

    pub fn extract<'a>(&self, sequence: &'a str) -> Option<Vec<&'a str>> {
        if let Some(captures) = self.matcher.captures(sequence) {
            Some(self.capture_group_names.iter().filter_map(|capture_group_name| {
                captures.name(capture_group_name).map(|matched_group| matched_group.as_str())
            }).collect())
        } else {
            None
        }
    }

    pub fn capture_group_names(&self) -> &[String] {
        &self.capture_group_names
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BarcodeError {
    #[error("barcode-matching regex (\"{0}\") has no capture groups")]
    RegexMissingCaptures(String),
    #[error("invalid regex")]
    InvalidRegex {
        #[from]
        source: regex::Error
    },
}

