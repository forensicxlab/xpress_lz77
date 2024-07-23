# xpress lz77 decompression algorithms
xpress LZ77 Plain and xpress LZ77+Huffman Decompression algorithms : A rust implementation using pyo3.

# Context
Both algorithms are used by microsoft and can help with digital forensics:

- Windows 1.X prefetch files
- Windows Hibernation

The use of pyo3 make the creation of a python package possible for integration in python3 tools where decompression performances are required.

## Use cases

- https://www.forensicxlab.com/posts/prefetch/
- https://www.forensicxlab.com/posts/hibernation/


# References

- Pseudo code algorithm : https://winprotocoldoc.blob.core.windows.net/productionwindowsarchives/MS-XCA/%5bMS-XCA%5d.pdf [Section 2.2]
