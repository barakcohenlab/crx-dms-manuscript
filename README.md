This repository includes code for the analyses described in:

**"Mutational scanning of *CRX* classifies clinical variants and reveals biochemical properties of the transcriptional effector domain"**

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

