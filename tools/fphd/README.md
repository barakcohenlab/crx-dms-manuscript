# Fast Parallel Hamming Distance

This is a small python library designed for computing many Hamming distances in parallel. It is implemented in the [Rust programming language](https://www.rust-lang.org/), but compiles to a Python-compatible library using [PyO3](https://pyo3.rs/).

## Usage

The code includes some comments documenting what each function does. The actual Hamming distance calculation is implemented in `hamming_threshold`. This function is currently exposed to the Python end-user through several interfaces, including `nearby_within_threshold`. Each of the functions is implemented to yield results like a Python generator. Here is an example of using `nearby_within_threshold` to compare a list of sequenced barcodes against a list of expected barcodes.

```python

expected_barcodes = [... some list of expected barcodes ...]
unassigned_barcodes = [... some list of barcodes that were not exact matches to an expected barcode ...]

barcode_correction_map = {}

for unassigned_barcode, corrections in zip(unassigned_barcodes, fphd.nearby_within_threshold(unassigned_barcodes, expected_barcodes, 4, False)):
    if len(corrections) == 1:
        expected_barcode, distance = corrections[0]
        barcode_correction_map[unassigned_barcode] = expected_barcode
```

The above code iterates through the barcodes in `unassigned_barcodes`, one at a time. For each unassigned barcode, the Hamming distance to all barcodes in `expected_barcodes` is calculated. Any expected barcode with a distance less than the specified threshold (`4`) will be returned, along with the distance (as a list of tuples of the form `(expected_barcode, distance)`). If the length of the returned list is one, that means there is only a single barcode in `expected_barcodes` that is within the specified threshold, in which case the association is written to a dictionary object.

## Specifing the number of threads to use for parallel Hamming distance computations

You should run `set_available_threads` once, after importing the `fphd` module, to specify the number of CPU cores to use for parallel Hamming distance calculations. For example:

```python
import fphd
fphd.set_available_threads(int(os.environ["SLURM_CPUS_PER_TASK"]))
```

The above sets the available threads to the environment variable `SLURM_CPUS_PER_TASK`.

## Potential for future improvements

It is likely that the performance of `hamming_threshold` can be improved, perhaps in part by reducing overhead at the Rust-Python foreign function interface.
