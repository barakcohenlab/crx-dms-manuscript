use ::pyo3::prelude::*;
use ::rayon::prelude::*;

#[pymodule]
#[pyo3(name = "fphd")]
fn module(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(set_available_threads, module)?)?;
    module.add_function(wrap_pyfunction!(nearby_within_threshold, module)?)?;
    module.add_function(wrap_pyfunction!(graph_statistics, module)?)?;
    module.add_function(wrap_pyfunction!(distances, module)?)?;
    Ok(())
}

/// Use the specified number of threads for parallel operations. This should be set once, prior to
/// calling any other fphd functions.
#[pyfunction]
fn set_available_threads(num_threads: usize) -> PyResult<()> {
    if let Err(_) = rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global() {
        return Err(pyo3::exceptions::PySystemError::new_err("Could not initialize thread pool for parallel operations."));
    }
    Ok(())
}

// FIXME: Can the speed of these be improved by using the raw python objects (e.g. PyUnicode)
// rather than the rust types (which incur a conversion penalty)?

fn hamming_threshold(str_a: &[u8], str_b: &[u8], threshold: usize) -> Option<usize> {
    let mut distance = 0;
    for (char_a, char_b) in str_a.iter().zip(str_b.iter()) {
        if char_a != char_b {
            distance += 1;
            if distance > threshold {
                return None;
            }
        }
    }
    Some(distance)
}

#[pyclass]
pub struct NearbyWithinThreshold {
    strings: std::vec::IntoIter<String>,
    targets: Vec<String>,
    threshold: usize,
    ignore_exact_match: bool,
}

#[pymethods]
impl NearbyWithinThreshold {
    fn __iter__(_self: PyRef<'_, Self>) -> PyRef<'_, Self> {
        _self
    }

    fn __next__(mut _self: PyRefMut<'_, Self>) -> Option<Vec<(String, usize)>> {
        let local_threshold = _self.threshold;
        let local_ignore_exact_match = _self.ignore_exact_match;
        _self.strings.next().map(|string| {
            (&_self.targets).par_iter().filter_map(|candidate_match| {
                match hamming_threshold(string.as_bytes(), candidate_match.as_bytes(), local_threshold) {
                    Some(distance) => {
                        if local_ignore_exact_match && distance == 0 {
                            None
                        } else {
                            Some((candidate_match.to_string(), distance))
                        }
                    },
                    None => None
                }
            }).collect()
        })
    }

    fn __len__(_self: PyRef<'_, Self>) -> usize {
        _self.strings.len()
    }
}

/// For each string in `strings`, return all strings in `targets` that are within a Hamming
/// distance of `threshold`, as a list of tuples of the form `(target_string, distance)`. If
/// `ignore_exact_match` is true, exact matches (e.g. Hamming distance of zero) will be ignored.
#[pyfunction]
#[pyo3(text_signature = "(strings, targets, threshold, ignore_exact_match, /)")]
fn nearby_within_threshold<'a, 'b>(strings: Vec<String>, targets: Vec<String>, threshold: usize, ignore_exact_match: bool) -> NearbyWithinThreshold {
    NearbyWithinThreshold {
        strings: strings.into_iter(),
        targets,
        threshold,
        ignore_exact_match
    }
}

#[pyclass]
pub struct GraphStatistics {
    #[pyo3(get)]
    strings: Vec<String>,
    #[pyo3(get)]
    eccentricities: Vec<usize>,
    #[pyo3(get)]
    radius: usize,
    #[pyo3(get)]
    diameter: usize
}

/// Compute pairwise hamming distances on `strings` and return various Hamming graph statistics.
#[pyfunction]
#[pyo3(text_signature = "(strings, threshold, /)")]
fn graph_statistics<'a, 'b>(strings: Vec<String>, threshold: usize) -> GraphStatistics {
    let eccentricities: Vec<usize> = strings.iter().map(|string_a| {
        strings.par_iter().map(|string_b| {
            hamming_threshold(string_a.as_bytes(), string_b.as_bytes(), threshold).unwrap_or(threshold)
        }).max().unwrap_or(0)
    }).collect();
    let radius = *eccentricities.iter().min().unwrap_or(&0);
    let diameter = *eccentricities.iter().max().unwrap_or(&0);
    GraphStatistics {
        strings,
        eccentricities,
        radius,
        diameter
    }
}

#[pyclass]
pub struct Distances {
    strings_a: std::vec::IntoIter<String>,
    strings_b: Vec<String>,
    threshold: usize,
}

#[pymethods]
impl Distances {
    fn __iter__(_self: PyRef<'_, Self>) -> PyRef<'_, Self> {
        _self
    }

    fn __next__(mut _self: PyRefMut<'_, Self>) -> Option<Vec<Option<usize>>> {
        let local_threshold = _self.threshold;
        _self.strings_a.next().map(|string_a| {
            (&_self.strings_b).par_iter().map(|string_b| {
                hamming_threshold(string_a.as_bytes(), string_b.as_bytes(), local_threshold)
            }).collect()
        })
    }

    fn __len__(_self: PyRef<'_, Self>) -> usize {
        _self.strings_a.len()
    }
}

/// For each string in `strings_a`, compute the Hamming distance to every string in `strings_b` and
/// return a list of these distances. If the distance is greater than `threshold`, `None` will be
/// returned for that pair of strings.
#[pyfunction]
#[pyo3(text_signature = "(strings_a, strings_b, threshold, /)")]
fn distances<'a, 'b>(strings_a: Vec<String>, strings_b: Vec<String>, threshold: usize) -> Distances {
    Distances {
        strings_a: strings_a.into_iter(),
        strings_b,
        threshold
    }
}
