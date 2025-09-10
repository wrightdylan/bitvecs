//! ## Bit Vector
//! A simple implementation of a vector with bitwise operation.
//! Second revision for improved memory management, faster performance, and
//! expanded functionality.

use std::fmt;
use std::ops;

pub struct BitVec {
    data:     Vec<u8>,  // data vector
    len:      usize,    // length in bits
    byte_idx: usize,    // current byte, used for sequential reading
    bit_idx:  u8,       // current bit, used for sequential reading
}

impl BitVec {
    // ########################################################################
    // Main functions
    // ########################################################################

    /// Constructs a new, empty, BitVec.
    /// 
    /// # Examples
    ///
    /// ```
    /// use bitvecs::BitVec;
    /// 
    /// let mut bv = BitVec::new();
    /// ```
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            len: 0,
            byte_idx: 0,
            bit_idx: 0,
        }
    }

    /// Constructs a new, empty, BitVec, with at least the specified capacity.
    /// 
    /// # Examples
    ///
    /// ```
    /// use bitvecs::BitVec;
    /// 
    /// let mut bv = BitVec::with_capacity(24);
    /// ```
    pub fn with_capacity(bits: usize) -> Self {
        let bytes = bits.div_ceil(8);

        Self {
            data: Vec::with_capacity(bytes),
            len: bits,
            byte_idx: 0,
            bit_idx: 0,
        }
    }

    /// Exports the BitVec data to binary format
    pub fn export(&self) -> String {
        match String::from_utf8(self.data.clone()) {
            Ok(s) => return s,
            Err(e) => panic!("{}", e),
        }
    }

    /// Completely fill the BitVec with either true or false according to
    /// the length of the vector
    pub fn fill(&mut self, value: bool) {
        let num_bytes = self.data.len();
        if value {
            self.data = vec![0xFF; num_bytes];
            let offset = self.len % 8;
            if offset != 0 {
                self.data[num_bytes - 1] <<= 8 - offset;
            }
        } else {
            self.data = vec![0; num_bytes];
        }
    }

    /// Generate a new BitVec from an array or other bit stream
    /// 
    /// # Examples
    ///
    /// ```
    /// use bitvecs::BitVec;
    /// 
    /// let array_of_bytes = [24, 51, 67];
    /// let mut bundle = BitVec::from(&array_of_bytes);
    /// ```
    pub fn from(data: &[u8]) -> Self {
        let len = data.len() * 8;

        Self {
            data: data.to_vec(),
            len,
            byte_idx: 0,
            bit_idx: 0,
        }
    }

    /// Import a BitVec from a string
    /// 
    /// # Examples
    ///
    /// ```
    /// use bitvecs::BitVec;
    /// 
    /// let string = "A test string".to_string();
    /// let mut bundle = BitVec::from_string(&string);
    /// ```
    pub fn from_string(data: &String) {} // Finish this

    /// Returns the bit value at the desired index. Bit is read from MSB
    pub fn get_bit(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let bit_index = index % 8;

        if index >= self.len {
            panic!("BitVec: index out of bounds")
        }

        let byte = self.data[byte_index];
        (byte & (1 << 7 - bit_index)) != 0
    }

    /// Get the bit index (typically used for reading)
    pub fn get_bit_idx(&self) -> u8 {
        self.bit_idx
    }

    /// Get the byte index (typically used for reading)
    pub fn get_byte_idx(&self) -> usize {
        self.byte_idx
    }

    /// Get current reading position in bits
    pub fn get_read_position(&self) -> usize {
        self.byte_idx * 8 + self.bit_idx as usize
    }

    /// Get the current capacity in bytes
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Get the number of bits stored in the vector
    pub fn len_bits(&self) -> usize {
        self.len
    }

    /// Build a bit mask of a given number of bits starting from the MSB
    pub fn mask_msb(size: usize) -> u8 {
        if size > 0 {
            0xFF << 8 - size
        } else {
            0xFF
        }
    }

    /// Build a bit mask of a given number of bits starting from the LSB
    pub fn mask_lsb(size: usize) -> u8 {
        if size > 0 {
            0xFF >> 8 - size
        } else {
            0xFF
        }
    }

    /// Removes and returns the last bit in the vector
    pub fn pop_bit(&mut self) -> Option<bool> {
        if self.len == 0 {
            return None;
        }

        let bit = self.get_bit(self.len - 1);
        self.set_bit(self.len - 1, false);

        // Update indices
        if self.bit_idx == 0 {
            if self.byte_idx > 0 {
                self.byte_idx -= 1;
                self.bit_idx = 7;
            }
        } else {
            self.bit_idx -= 1;
        }
        
        self.len -= 1;
        Some(bit)
    }

    /// Removes and returns the last contiguous byte in the vector regardless
    /// of position. If there are less than 8 bits, then it returns what is
    /// available, with the remainder set as 0s.
    pub fn pop_byte(&mut self) -> Option<u8> {
        let last_byte = self.data.pop();

        if self.len < 8 {
            self.len = 0;
            return last_byte;
        } else {
            let len_tail = self.len % 8;
            self.len -= 8;
            // When there is perfect byte alignment
            if len_tail == 0 {
                return last_byte;
            }
            let len_head = 8 - len_tail;
            let tail = last_byte.unwrap() >> len_head;
            
            let last_index = self.data.len() - 1;
            let head = self.data[last_index] << len_tail;
            self.data[last_index] &= 0xFF << len_head;

            return Some(head | tail);
        }
    }

    /// Deprecated as the name is a bit ambiguous
    #[deprecated(since = "0.1.1", note = "This function is deprecated, please use the pop_vec_byte function instead.")]
    pub fn pop_full_byte(&mut self) -> Option<u8> {
        self.pop_vec_byte()
    }

    /// Removes and returns the last byte from the vector.
    pub fn pop_vec_byte(&mut self) -> Option<u8> {
        let truncation = self.len % 8;
        self.len -= truncation;
        // handle bit read position? If bytes are being popped, the vector
        // probably isn't being used for sequential read

        self.data.pop()
    }
    
    /// Pushes a bit to the vector
    pub fn push_bit(&mut self, value: bool) {
        let byte_offset = self.len / 8;
        let bit_offset = (self.len % 8) as u8;

        // Last byte in vector is already full
        if bit_offset == 0 {
            self.data.push(0);
        }

        // Only need to set the bit if value is true
        if value {
            self.data[byte_offset] |= 1 << 7 - bit_offset;
        }
        self.len += 1;
    }

    /// Pushes a byte to the vector
    pub fn push_byte(&mut self, byte: u8) {
        let byte_offset = self.len / 8;
        let bit_offset = (self.len % 8) as u8;

        // When there is perfect byte alignment
        if bit_offset == 0 {
            self.data.push(byte);
        } else {
            self.data[byte_offset] |= byte >> bit_offset;
            self.data.push(byte << 8 - bit_offset);
        }

        self.len += 8;
    }

    /// Deprecated as the name is too similar to a new function
    #[deprecated(since = "0.1.0", note = "This function is deprecated, please use the seq_read function instead.")]
    pub fn read_bit(&mut self) -> Option<u8> {
        self.seq_read()
    }

    /// This is a sequential read which increments the bit index, not a return of the bit value at a specific index.
    /// For the latter functionality use get_bit().
    pub fn seq_read(&mut self) -> Option<u8> {
        if self.byte_idx >= (self.len + 7) / 8 {
            return None;
        }

        let bit = (self.data[self.byte_idx] >> (7 - self.bit_idx)) & 1;
        self.bit_idx += 1;
        if self.bit_idx == 8 {
            self.byte_idx += 1;
            self.bit_idx = 0;
        }

        Some(bit)
    }

    /// Reads 8 bits in sequence and returns a byte (can be offset)
    pub fn read_byte(&mut self) -> Option<u8> {
        let mut byte: u8 = 0;
        for _ in 0..8 {
            if let Some(bit) = self.seq_read() {
                byte = (byte << 1) | bit;
            } else {
                return None;
            }
        }
        Some(byte)
    }

    /// Reset sequential reading position
    pub fn reset_seq_read(&mut self) {
        self.byte_idx = 0;
        self.bit_idx = 0;
    }

    /// Finds the next set bit in a BitVec from a start index and returns
    /// the index of that bit if one is found. Most useful for one-hot encoding.
    pub fn next_set_bit(&self, start_idx: usize) {} // Finish this

    /// Sets the bit at the desired index. If the bit to be set is beyond the
    /// current capacity, then the vector will grow to accomodate the new bit
    /// and fill the gap with 0s rather than panic
    pub fn set_bit(&mut self, index: usize, value: bool) {
        let byte_index = index / 8;
        let bit_index = index % 8;

        while self.data.len() <= byte_index {
            self.data.push(0);
        }

        if value {
            self.data[byte_index] |= 1 << 7 - bit_index;
        } else {
            self.data[byte_index] &= !(1 << 7 - bit_index);
        }
    }

    /// Set reading position in bits
    pub fn set_read_position(&mut self, bit_idx: usize) -> bool {
        if bit_idx >= self.len {
            return false;
        }
        self.byte_idx = bit_idx / 8;
        self.bit_idx = (bit_idx % 8) as u8;
        true
    }

    /// Checks if all bytes are zero
    pub fn is_zero(&self) -> bool {
        for byte_idx in 0..self.data.len() {
            if self.data[byte_idx] != 0 {
                return false;
            }
        }

        true
    }

    // ########################################################################
    // Display functions (for debugging)
    // ########################################################################

    /// Converts the data to a binary representation
    pub fn as_binary(&self) -> String {
        self.data.iter()
            .map(|byte| format!("{:08b}", byte))
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// Converts the data to a readable format
    pub fn as_char(&self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }

    // ########################################################################
    // Operator functions
    // ########################################################################

    /// Concatenates two (and only two) BitVecs, creating a new BitVec
    pub fn concat(&self, other: &BitVec) -> Self {
        let bit_offset = self.len % 8;

        // If we happen to have perfect alignment
        if bit_offset == 0 {
            return BitVec {
                data: [self.data.clone(), other.data.clone()].concat(),
                len: self.len + other.len,
                byte_idx: 0,
                bit_idx: 0
            };
        } else {
            let mut new_bitvec = BitVec { data: self.data.clone(), len: self.len.clone(), byte_idx: 0, bit_idx: 0 };
            new_bitvec.extend(other);

            return new_bitvec;
        }
    }

    /// Extends one BitVec with another
    pub fn extend(&mut self, other: &BitVec) {
        let bit_offset = self.len % 8;

        // If we happen to have perfect alignment
        if bit_offset == 0 {
            self.data.extend(&other.data);
            self.len += other.len;
        } else {
            let mut bits_remaining = other.len;
            let mut byte_index = 0;
            while bits_remaining >= 8 {
                self.push_byte(other.data[byte_index]);
                bits_remaining -= 8;
                byte_index += 1;
            }
            // last or only byte
            let byte = other.data.last().copied().unwrap();
            let diff = 8 - bit_offset;
            if let Some(last_byte) = self.data.last_mut() {
                *last_byte |= byte >> bit_offset;
                bits_remaining -= diff;
                self.len += diff;
            }
            if bits_remaining > 0 {
                self.data.push(byte << diff);
                self.len += bits_remaining;
            }
        }
    }

    /// Returns a new BitVector containing the complimentary intersection
    /// between two BitVectors (aka NAND). This is likely to be one of the most
    /// common compound operation, so it gets its own special function.
    pub fn comp_int(&self, other: &BitVec) {} // Finish this!

    /// Returns a new BitVector containing the similarities between two BitVecs (aka AND)
    pub fn intersec(&self, other: &BitVec) {} // Finish this!

    /// Returns a new BitVector containing an inversion of the original (aka NOT)
    pub fn compliment(&self) -> BitVec {
        let mut inverted = Vec::new();

        for byte in self.data.iter() {
            inverted.push(!byte);
        }

        // Everything got inverted, so the 'unset' bits need to be reset to 0
        if let Some(last_byte) = inverted.last_mut() {
            *last_byte &= BitVec::mask_msb(self.len % 8);
        }

        Self {
            data: inverted,
            len: self.len,
            byte_idx: 0,
            bit_idx: 0,
        }
    }

    /// Returns a new BitVector containing the symmetric difference between two BitVecs (aka XOR)
    pub fn symm_diff(&self, other: &BitVec) {} // Finish this!

    /// Returns a new BitVector containing a union of two BitVecs (aka OR)
    pub fn union(&self, other: &BitVec) {} // Finish this!
}

// ############################################################################
// Custom fmt
// ############################################################################

impl fmt::Display for BitVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BitVec {{ data: {:?}, len: {}, byte_idx: {}, bit_idx: {} }}",
            self.data, self.len, self.byte_idx, self.bit_idx
        )
    }
}

// ############################################################################
// Custom ops
// Logic operators will not reset the read position, so depending on order, it
// may be out of bounds.
// ############################################################################

impl ops::Add for BitVec {
    type Output = Self;

    fn add(self, rhs: BitVec) -> Self::Output {
        self.concat(&rhs)
    }
}

impl ops::AddAssign for BitVec {
    fn add_assign(&mut self, rhs: Self) {
        self.extend(&rhs);
    }
}

// todo
// BitAnd
// BitAndAssign
// BitOr
// BitOrAssign
// BitXor
// BitXorAssign

/// Returns the byte at index
impl ops::Index<usize> for BitVec {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

/// Modifies the byte at index
/// This can cause a mismatch with the bit length if extra 1s are added where
/// they should not be
impl ops:: IndexMut<usize> for BitVec {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

/// Performs bitwise NOT operation on the data element of BitVec
impl ops::Not for BitVec {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.compliment()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_correct_bit() {
        let test_byte = BitVec::from(&[128, 1]);
        assert_eq!(test_byte.get_bit(0), true);
        assert_eq!(test_byte.get_bit(7), false);
        assert_eq!(test_byte.get_bit(8), false);
        assert_eq!(test_byte.get_bit(15), true);
    }

    #[test]
    fn set_correct_bits() {
        let mut bv = BitVec::from(&[0]);
        bv.set_bit(0, true);
        assert_eq!(bv.data[0], 128u8);
        bv.set_bit(7, true);
        assert_eq!(bv.data[0], 129u8);
        bv.set_bit(0, false);
        assert_eq!(bv.data[0], 1u8);
        bv.set_bit(20, true); // byte 2 = 0b00001000
        assert_eq!(bv.data[2], 8u8);
    }

    #[test]
    fn pop_last_byte() {
        // test vector: 0b01101010 0b101xxxxx (106, 160)
        // len = 11
        // expected result = 0b01010101 (85)
        let mut bv = BitVec { data: vec![106, 160], len: 11, byte_idx: 0, bit_idx: 0};
        assert_eq!(bv.pop_byte().unwrap(), 85u8);
        assert_eq!(bv.len, 3);
        // remainder = 0b011xxxxx (96)
        assert_eq!(bv.pop_byte().unwrap(), 96u8);
    }

    #[test]
    fn push_bits() {
        // Start with a byte of 0b00000010 (2)
        let mut bv = BitVec { data: vec![2], len: 7, byte_idx: 0, bit_idx: 0};
        bv.push_bit(true);
        assert_eq!(bv.get_bit(7), true);
        assert_eq!(bv.len, 8);
        bv.push_bit(true);
        assert_eq!(bv.data[1], 128_u8);
    }

    #[test]
    fn fill_vectors() {
        let mut bv1 = BitVec { data: vec![0, 0], len: 12, byte_idx: 0, bit_idx: 0};
        bv1.fill(true);
        assert_eq!(bv1.data[1], 240);
        let mut bv2 = BitVec { data: vec![0, 0], len: 16, byte_idx: 0, bit_idx: 0};
        bv2.fill(true);
        assert_eq!(bv2.data[1], 255);
    }

    #[test]
    fn extend_bitvec() {
        // bv1 = 0b00001xxx
        let mut bv1 = BitVec { data: vec![0x08], len: 5, byte_idx: 0, bit_idx: 0};
        // bv2 = 0b1011xxxx
        let bv2 = BitVec { data: vec![0xB0], len: 4, byte_idx: 0, bit_idx: 0};
        // bv1 = 0b00001101 1xxxxxxx
        bv1 += bv2;
        assert_eq!(bv1.data[0], 13);
        assert_eq!(bv1.data[1], 128);
        assert_eq!(bv1.len, 9);
    }

    #[test]
    fn new_from_add() {
        // bv1 = 0b00001xxx
        let bv1 = BitVec { data: vec![0x08], len: 5, byte_idx: 0, bit_idx: 0};
        // bv2 = 0b1011xxxx
        let bv2 = BitVec { data: vec![0xB0], len: 4, byte_idx: 0, bit_idx: 0};
        // bv1 = 0b00001101 1xxxxxxx
        let bv3 = bv1 + bv2;
        assert_eq!(bv3.data[0], 13);
        assert_eq!(bv3.data[1], 128);
        assert_eq!(bv3.len, 9);
    }

    #[test]
    fn inverted_bytes() {
        // bv1 = 0b01101101
        let bv1 = BitVec { data: vec![0x6D], len: 8, byte_idx: 0, bit_idx: 0};
        // bv2 = 0b10011xxx
        let bv2 = BitVec { data: vec![0x98], len: 5, byte_idx: 0, bit_idx: 0};
        assert_eq!(bv1.compliment().data[0], 0x92);
        assert_eq!(bv2.compliment().data[0], 0x60);
    }
}