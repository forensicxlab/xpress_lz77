// Huffman+LZ77 Decompression Algorithm
// Reference: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/a8b7cb0a-92a6-4187-a23b-5e14273b96f8

use std::cell::RefCell;
use std::fmt;
use std::io::Error;
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct PrefixCodeNode {
    id: u32,
    symbol: u32,
    leaf: bool,
    child: [Option<Rc<RefCell<PrefixCodeNode>>>; 2],
}

#[derive(Default, Clone)]
pub struct PrefixCodeSymbol {
    id: u32,
    symbol: u32,
    length: u8,
}

pub struct PrefixCodeTree {
    tree: Vec<Rc<RefCell<PrefixCodeNode>>>,
    symbols: Vec<PrefixCodeSymbol>,
}

impl PrefixCodeTree {
    fn new() -> Result<Self, Error> {
        Ok(PrefixCodeTree {
            symbols: vec![PrefixCodeSymbol::default(); 512],
            tree: (0..1024)
                .map(|_| Rc::new(RefCell::new(PrefixCodeNode::default())))
                .collect(),
        })
    }

    fn construct_symbols(&mut self, input: &[u8]) {
        for i in 0..256 {
            let mut value = input[i];

            self.symbols[2 * i].id = (2 * i) as u32;
            self.symbols[2 * i].symbol = (2 * i) as u32;
            self.symbols[2 * i].length = value & 0xF;

            value >>= 4;

            self.symbols[2 * i + 1].id = (2 * i + 1) as u32;
            self.symbols[2 * i + 1].symbol = (2 * i + 1) as u32;
            self.symbols[2 * i + 1].length = value & 0xF;
        }

        self.symbols
            .sort_by(|a, b| a.length.cmp(&b.length).then(a.symbol.cmp(&b.symbol)));
    }

    fn construct_tree(&mut self) {
        let mut i = 0;
        while i < 512 && self.symbols[i].length == 0 {
            i = i + 1;
        }

        let mut mask = 0;
        let mut bits = 1;

        let mut leaf_index = 1;

        while i < 512 {
            self.tree[leaf_index].borrow_mut().id = leaf_index as u32;
            self.tree[leaf_index].borrow_mut().symbol = self.symbols[i].symbol;
            self.tree[leaf_index].borrow_mut().leaf = true;
            mask = mask << (self.symbols[i].length - bits);
            bits = self.symbols[i].length;

            let mut index = leaf_index + 1;

            let mut node = Rc::clone(&self.tree[0]);

            let mut counter = bits;
            while counter > 1 {
                counter -= 1;
                let child_index = ((mask >> counter) & 1) as usize;
                if node.borrow().child[child_index].is_none() {
                    node.borrow_mut().child[child_index] = Some(Rc::clone(&self.tree[index]));
                    self.tree[index].borrow_mut().leaf = false;
                    index += 1;
                }
                let child = Rc::clone(node.borrow().child[child_index].as_ref().unwrap());
                node = child;
            }
            node.borrow_mut().child[mask & 1] = Some(Rc::clone(&self.tree[leaf_index]));
            leaf_index = index;
            mask += 1;
            i += 1;
        }
    }

    fn decode_symbol(&self, bstr: &mut BitStream) -> Result<u32, Error> {
        let mut current_node = Rc::clone(&self.tree[0]);
        loop {
            let bit = bstr.lookup(1);
            bstr.skip(1)?;
            let child_option = current_node.borrow().child[bit as usize].clone();
            if let Some(child) = child_option {
                if child.borrow().leaf {
                    return Ok(child.borrow().symbol);
                } else {
                    current_node = Rc::clone(&child);
                }
            } else {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Corruption detected",
                ));
            }
        }
    }
}

impl fmt::Debug for PrefixCodeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Node {}: symbol {} leaf {}\n",
            self.id, self.symbol, self.leaf
        )
    }
}

impl fmt::Debug for PrefixCodeSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Symbol {}: symbol {} length {}\n",
            self.id, self.symbol, self.length
        )
    }
}

struct BitStream<'a> {
    source: &'a [u8],
    index: usize,
    mask: u32,
    bits: u32,
}

impl<'a> BitStream<'a> {
    fn new(source: &'a [u8], in_pos: usize) -> Result<Self, Error> {
        let low_mask = u16::from_le_bytes([source[in_pos], source[in_pos + 1]]);
        let mut final_mask = (low_mask as u32) << 16;
        let high_mask = u16::from_le_bytes([source[in_pos + 2], source[in_pos + 3]]);
        final_mask += high_mask as u32;
        Ok(BitStream {
            source,
            index: in_pos + 4,
            mask: final_mask,
            bits: 32,
        })
    }

    fn lookup(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        return self.mask >> (32 - n);
    }

    fn skip(&mut self, n: u32) -> Result<(), Error> {
        self.mask <<= n;
        self.bits -= n;
        if self.bits < 16 {
            if self.index + 2 > self.source.len() {
                return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "EOF Error"));
            }
            let shift: u32 =
                u16::from_le_bytes([self.source[self.index], self.source[self.index + 1]]) as u32;
            self.mask = (self.mask + (shift << (16 - self.bits))) & 0xFFFFFFFF;
            self.index += 2;
            self.bits += 16;
        }
        Ok(())
    }
}

fn lz77_huffman_decompress_chunk(
    in_idx: usize,
    input: &[u8],
    out_idx: usize,
    output: &mut Vec<u8>,
    chunk_size: usize,
) -> Result<(usize, usize), Error> {
    if in_idx + 256 > input.len() {
        return Err(Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Unexpected Eof",
        ));
    }

    let mut prefix_code_tree: PrefixCodeTree = PrefixCodeTree::new()?;
    prefix_code_tree.construct_symbols(&input[in_idx..]);
    prefix_code_tree.construct_tree();

    let mut bstr = BitStream::new(input, in_idx + 256)?;
    let mut i = out_idx;
    while i < out_idx + chunk_size {
        let symbol = match prefix_code_tree.decode_symbol(&mut bstr) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        if symbol < 256 {
            output[i] = symbol as u8;
            i += 1;
        } else {
            let mut symbol = symbol as usize - 256;
            let mut length = symbol & 15;
            symbol >>= 4;

            let mut offset = if symbol != 0 {
                bstr.lookup(symbol as u32) as isize
            } else {
                0
            };

            offset |= 1 << symbol;
            offset = -offset;

            if length == 15 {
                length = input[bstr.index] as usize + 15;
                bstr.index += 1;
                if length == 270 {
                    length =
                        u16::from_le_bytes([input[bstr.index], input[bstr.index + 1]]) as usize;
                    bstr.index += 2;
                }
            }

            bstr.skip(symbol as u32)?;

            length += 3;
            while length > 0 {
                if (i as isize) + offset < 0 {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Can't decompress the data",
                    ));
                }

                output[i] = output[(i as isize + offset) as usize];
                i += 1;
                length -= 1;
                if length == 0 {
                    break;
                }
            }
        }
    }

    Ok((bstr.index, i))
}

pub fn lz77_huffman_decompress(input: &[u8], output_size: usize) -> Result<Vec<u8>, Error> {
    let mut output = vec![0u8; output_size];
    let mut in_idx = 0;
    let mut out_idx = 0;

    loop {
        let chunk_size = if output_size - out_idx > 65536 {
            65536
        } else {
            output_size - out_idx
        };

        let result = lz77_huffman_decompress_chunk(in_idx, input, out_idx, &mut output, chunk_size);
        let (new_in_idx, new_out_idx) = match result {
            Ok((new_in_idx, new_out_idx)) => (new_in_idx, new_out_idx),
            Err(err) => return Err(err),
        };

        in_idx = new_in_idx;
        out_idx = new_out_idx;

        if out_idx >= output.len() || in_idx >= input.len() {
            break;
        }
    }

    Ok(output)
}
