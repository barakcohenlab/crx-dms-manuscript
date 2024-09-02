This repository includes code for the analyses described in:

**"Mutational scanning of *CRX* classifies clinical variants and reveals biochemical properties of the transcriptional effector domain"** ([10.1101/gr.279415.124](https://doi.org/10.1101/gr.279415.124))

James L. Shepherdson¹﹐²\
David M. Granas¹﹐²\
Jie Li¹﹐²\
Zara Shariff¹﹐²\
Stephen P. Plassmeyer³﹐⁴\
Alex S. Holehouse³﹐⁴\
Michael A. White¹﹐²\
Barak A. Cohen¹﹐²^

¹Department of Genetics,\
²Edison Family Center for Genome Sciences & Systems Biology,\
³Department of Biochemistry and Molecular Biophysics,\
⁴Center for Molecular Condensates,\
Washington University in St. Louis School of Medicine, St. Louis, MO 63110, USA

\^ Correspondence: Barak A. Cohen <cohen@wustl.edu>

## Description

- **DMS_analysis**: includes code and scripts for analysis of barcode abundance data from sorted fractions to compute variant activity scores and generate figures
    * `call_variants.py` generates a barcode-to-variant map from PacBio long read sequencing data to associate sequence barcodes with coding variants
- **tools**: includes software tools developed for this manuscript that are used in other scripts:
    * **bcbuddy** is used to extract barcodes from sequencing reads
    * **dms_tools** includes a library for parsing the output of `minimap2` for long read data analysis; used by `call_variants.py`
    * **fphd** implements a parallelized Hamming distance calculation for barcode error correction

## Notes

`crx_genomic_positions.tsv` is provided to translate protein-level variant coordinates into cDNA and gDNA coordinates. To produce a VCF-compatible file, `awk` or a similar tool can be used. For example:

```
awk 'BEGIN {FS="\t"; OFS="\t"} {print 19, $5, ".", $6, $7, ".", "PASS", "var="$1$2$3}' crx_genomic_positions.tsv
```

Note that you may wish to first filter the `crx_genomic_positions` table to particular variants of interest, or join it with the DMS or computational predictor data to include scores in the VCF.
