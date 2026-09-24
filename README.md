# u256

A 256-bit unsigned integer in four 64-bit limbs, for EVM amounts, with checked
arithmetic and hex parsing.

Rust has `u128`, which is not enough: a single wei balance in a realistic
contract exceeds it, and every ABI word is 256 bits. Pulling in a bignum crate
is usually the right answer in production, but it is a large dependency for a
tool that only needs add, subtract, multiply and divide by small integers.

## What is here

- `from_hex` / `to_hex` / `to_decimal`, tolerant of `0x`, mixed case and leading
  zeros, rejecting anything over 32 bytes.
- `checked_add`, `checked_sub`, `checked_mul_u64`, `checked_div_u64`. All return
  `Option`, never wrap. Silent overflow on a balance is the bug that costs money.
- Big-endian byte conversion, which is the layout ABI encoding uses.

## What is deliberately missing

- **No general multiplication or division.** Only by a `u64`. Full 256x256
  multiplication is a different amount of code and not needed to display or
  compare amounts.
- **No signed arithmetic.** EVM's `SDIV`/`SMOD` are rarely needed off-chain.
- **No modular exponentiation.** Use a dedicated crate for anything cryptographic.

## Development

```bash
cargo test
```

## License

MIT
