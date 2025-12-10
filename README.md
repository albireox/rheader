# rheader

A very simple, very fast library to read FITS headers.

This package provides a no-frills interface to read FITS headers from FITS files, either uncompressed or gzipped. It is a work in progress and at this point it provides minimal support for the FITS standard (for example, `CONTINUE`, `HIERARCH`, or `COMMENT` cards are not yet supported). Instead of providing a nice, ergonomic API, the focus is on speed and simplicity and the use-case is to be able to read tens of thousands of FITS headers as fast as possible.

`rheader` is written in Rust and provides Python bindings via PyO3.

## Installation

`rheader` is not yet in PyPI, but you can install it via `pip` from the GitHub repository:

```bash
pip install -U git+https://github.com/albireox/rheader.git
```

## Minimal Example

```python
>>> from rheader import read_header

>>> header = read_header('sdR-r2-00022684.fit.gz')
>>> type(header)
dict


>>> print(header)
{'SIMPLE': (True, 'conforms to FITS standard'),
 'BITPIX': (16, 'array data type'),
 'NAXIS': (2, 'number of array dimensions'),
 'NAXIS1': (4352, None),
 'NAXIS2': (4224, None),
 'V_ARCHON': ('0.9.0', None),
 'FILENAME': ('sdR-r2-00022684.fit.gz', 'File basename'),
 'EXPOSURE': (22684, 'Exposure number'),
 'SPEC': ('sp2', 'Spectrograph name'),
 'OBSERVAT': ('LCO', 'Observatory'),
 'TAI-BEG': (5205261151.193, 'MJD(TAI) seconds at start of integration'),
 'DATE-OBS': ('2023-10-29T01:52:31.193', 'Start of the observation'),
 'MJD': (60246, 'Modified Julian Date'),
 'REQTIME': (10.0, 'Requested exposure time'),
...}
```

## Available functions

Currently, `rheader` provides two functions: `read_header` which reads the header from a FITS file to a dictionary of tuples, and `read_header_to_class` which reads the header to a minimal `Header` class.

## Benchmarks

Benchmarks reading a directory with 62 gzipped FITS files using `rheader`, `astropy.io.fits`, and `fitsio`:

```python
In [10]: from rheader import read_header

In [11]: %timeit x = [read_header(str(f)) for f in files]
16.4 ms ± 72.5 μs per loop (mean ± std. dev. of 7 runs, 100 loops each)

In [12]: from astropy.io import fits

In [13]: %timeit x = [fits.getheader(str(f)) for f in files]
7.98 s ± 24.4 ms per loop (mean ± std. dev. of 7 runs, 1 loop each)

In [14]: import fitsio

In [15]: %timeit x = [fitsio.read_header(str(f)) for f in files]
19 s ± 15.6 ms per loop (mean ± std. dev. of 7 runs, 1 loop each)
```

`rheader` is about 500x faster than `astropy.io.fits` and about 1100x faster than `fitsio` when reading gzipped FITS files. This seems to be mainly due to the use of Rust's `flate2` crate for fast gzip file streaming.

The improvements are less dramatic when reading uncompressed FITS files, but `rheader` is still significantly faster:

```python
In [3]: from rheader import read_header

In [4]: %timeit x = [read_header(str(f)) for f in uncompressed_files]
14.3 ms ± 71.3 μs per loop (mean ± std. dev. of 7 runs, 100 loops each)

In [5]: from astropy.io import fits

In [6]: %timeit x = [fits.getheader(str(f)) for f in uncompressed_files]
39.9 ms ± 606 μs per loop (mean ± std. dev. of 7 runs, 10 loops each)

In [7]: import fitsio

In [8]: %timeit x = [fitsio.read_header(str(f)) for f in uncompressed_files]
98.2 ms ± 846 μs per loop (mean ± std. dev. of 7 runs, 10 loops each)
```

All benchmarks were run on an Ubuntu machine with 28 cores and 62 GB of RAM. The files were stored in an HDD disk.
