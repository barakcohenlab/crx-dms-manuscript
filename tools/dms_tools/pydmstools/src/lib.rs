use std::path;
use dms_tools::alignment;
use ::pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "dmstools")]
fn module(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<RecordReaderWrapper>()?;
    module.add_class::<RecordWrapper>()?;
    Ok(())
}

#[pyclass(name = "RecordReader")]
#[pyo3(text_signature = "(PAF_file_name, /)")]
pub struct RecordReaderWrapper {
    record_reader: alignment::RecordReader
}

#[pymethods]
impl RecordReaderWrapper {
    #[allow(non_snake_case)]
    #[new]
    pub fn new(PAF_file_name: &str) -> PyResult<Self> {
        let PAF_file_path = path::Path::new(PAF_file_name);

        Ok(RecordReaderWrapper {
            record_reader: alignment::RecordReader::read(PAF_file_path).map_err(|error| {
                pyo3::exceptions::PyFileNotFoundError::new_err(format!("records file error: {:?}", error))
            })?
        })
    }
}

#[pyproto]
impl pyo3::PyIterProtocol for RecordReaderWrapper {
    fn __iter__(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }

    fn __next__(mut self_: PyRefMut<Self>) -> PyResult<Option<RecordWrapper>> {
        match self_.record_reader.next() {
            Some(Ok(record)) => Ok(Some(record.into())),
            Some(Err(error)) => Err(pyo3::exceptions::PyValueError::new_err(format!("{}", error))),
            None => Ok(None)
        }
    }
}

#[pyclass(name = "Record")]
#[derive(Clone)]
pub struct RecordWrapper {
    record: alignment::Record
}

impl From<alignment::Record> for RecordWrapper {
    fn from(record: alignment::Record) -> Self {
        Self {
            record: record
        }
    }
}

#[pymethods]
impl RecordWrapper {
    #[getter]
    pub fn query_name(&self) -> PyResult<&str> {
        Ok(&self.record.query.name)
    }

    #[getter]
    pub fn mapping_quality(&self) -> PyResult<u8> {
        Ok(self.record.mapping_quality)
    }

    pub fn alignment(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.record.alignment))
    }

    pub fn call_coding_variants(&self, start_in_target: usize, end_in_target: usize) -> PyResult<Vec<(char, usize, char)>> {
        match self.record.alignment_subset(start_in_target, end_in_target) {
            Some(alignment) => {
                alignment.call_coding_variants().map_err(|error| {
                    pyo3::exceptions::PyValueError::new_err(format!("{}", error))
                })
            },
            None => {
                Err(pyo3::exceptions::PyIndexError::new_err(format!("alignment region not possible for this read")))
            }
        }
    }

    pub fn alignment_subset(&self, start_in_target: usize, end_in_target: usize) -> PyResult<String> {
        match self.record.alignment_subset(start_in_target, end_in_target) {
            Some(alignment) => {
                let (_, query) = alignment.make_sequences();
                Ok(query)
            },
            None => {
                Err(pyo3::exceptions::PyIndexError::new_err(format!("alignment region not possible for this read")))
            }
        }
    }
}
