//! A minimal 256-bit unsigned integer, sized for EVM arithmetic.
//!
//! Four little-endian 64-bit limbs. Enough to hold wei amounts, block numbers
//! and ABI words, with hex parsing that tolerates the `0x` prefix and leading
//! zeros. Checked operations return `None` instead of wrapping, because silent
//! overflow on a balance is the kind of bug that costs money.

const LIMBS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct U256(pub [u64; LIMBS]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    BadHex(usize),
    TooLong,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty input"),
            ParseError::BadHex(i) => write!(f, "non-hex character at {}", i),
            ParseError::TooLong => write!(f, "more than 32 bytes"),
        }
    }
}

impl std::error::Error for ParseError {}

impl U256 {
    pub const ZERO: U256 = U256([0; LIMBS]);

    /// Largest representable value, useful as a comparison sentinel.
    pub const MAX: U256 = U256([u64::MAX; LIMBS]);

    pub fn from_u64(v: u64) -> Self {
        U256([v, 0, 0, 0])
    }

    pub fn is_zero(&self) -> bool {
        self.0 == [0; LIMBS]
    }

    fn nibble(c: u8) -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    }

    /// Parse hex, with or without `0x`, lower or upper case.
    pub fn from_hex(input: &str) -> Result<Self, ParseError> {
        let s = input
            .strip_prefix("0x")
            .or_else(|| input.strip_prefix("0X"))
            .unwrap_or(input);
        if s.is_empty() {
            return Err(ParseError::Empty);
        }
        let bytes = s.as_bytes();
        // pad to an even number of nibbles so pairs line up
        let mut out = [0u8; 32];
        let digits = bytes.len();
        if digits > 64 {
            return Err(ParseError::TooLong);
        }
        for (i, c) in bytes.iter().enumerate() {
            let n = Self::nibble(*c).ok_or(ParseError::BadHex(i))?;
            let nibble_index = 64 - digits + i; // position from the left in 64 nibbles
            let byte_index = nibble_index / 2;
            if nibble_index % 2 == 0 {
                out[byte_index] = n << 4;
            } else {
                out[byte_index] |= n;
            }
        }
        Ok(Self::from_be_bytes(out))
    }

    /// Big-endian bytes, as they appear in ABI encoding.
    pub fn from_be_bytes(mut b: [u8; 32]) -> Self {
        b.reverse();
        let mut limbs = [0u64; LIMBS];
        for (i, chunk) in b.chunks_exact(8).enumerate() {
            limbs[i] = u64::from_le_bytes(chunk.try_into().unwrap());
        }
        U256(limbs)
    }

    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut out = [0u8; 32];
        for (i, limb) in self.0.iter().enumerate() {
            out[i * 8..i * 8 + 8].copy_from_slice(&limb.to_le_bytes());
        }
        out.reverse();
        out
    }

    /// Minimal hex, the way Ethereum renders quantities: no leading zero
    /// nibble, so `0xde0b...` rather than `0x0deb...`.
    pub fn to_hex(&self) -> String {
        let bytes = self.to_be_bytes();
        let first = bytes.iter().position(|b| *b != 0).unwrap_or(31);
        let hex: String = bytes[first..]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        let trimmed = if hex.len() > 1 && hex.starts_with('0') {
            &hex[1..]
        } else {
            &hex[..]
        };
        format!("0x{}", trimmed)
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        let mut out = [0u64; LIMBS];
        let mut carry = 0u128;
        for i in 0..LIMBS {
            let sum = self.0[i] as u128 + other.0[i] as u128 + carry;
            out[i] = sum as u64;
            carry = sum >> 64;
        }
        if carry != 0 {
            None
        } else {
            Some(U256(out))
        }
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        let mut out = [0u64; LIMBS];
        let mut borrow = 0i128;
        for i in 0..LIMBS {
            let diff = self.0[i] as i128 - other.0[i] as i128 - borrow;
            out[i] = diff as u64;
            borrow = if diff < 0 { 1 } else { 0 };
        }
        if borrow != 0 {
            None
        } else {
            Some(U256(out))
        }
    }

    /// Multiply by a small integer, checking for overflow.
    pub fn checked_mul_u64(self, m: u64) -> Option<Self> {
        let mut out = [0u64; LIMBS];
        let mut carry = 0u128;
        for i in 0..LIMBS {
            let prod = self.0[i] as u128 * m as u128 + carry;
            out[i] = prod as u64;
            carry = prod >> 64;
        }
        if carry != 0 {
            None
        } else {
            Some(U256(out))
        }
    }

    /// Divide by a small non-zero integer, returning floored quotient.
    pub fn checked_div_u64(self, d: u64) -> Option<Self> {
        if d == 0 {
            return None;
        }
        let mut out = [0u64; LIMBS];
        let mut rem = 0u128;
        for i in (0..LIMBS).rev() {
            let cur = (rem << 64) | self.0[i] as u128;
            out[i] = (cur / d as u128) as u64;
            rem = cur % d as u128;
        }
        Some(U256(out))
    }

    /// Decimal string, for display. Repeated division, slow but obvious.
    pub fn to_decimal(&self) -> String {
        if self.is_zero() {
            return "0".to_string();
        }
        let mut value = *self;
        let mut digits = Vec::new();
        while !value.is_zero() {
            let q = value.checked_div_u64(10).unwrap();
            let rem = value.checked_sub(q.checked_mul_u64(10).unwrap()).unwrap();
            digits.push(b'0' + rem.0[0] as u8);
            value = q;
        }
        digits.reverse();
        String::from_utf8(digits).unwrap()
    }
}

impl std::str::FromStr for U256 {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}
