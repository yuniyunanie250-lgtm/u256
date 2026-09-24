use u256::U256;

#[test]
fn parses_hex_with_and_without_prefix() {
    assert_eq!(U256::from_hex("0xff").unwrap(), U256::from_u64(255));
    assert_eq!(U256::from_hex("FF").unwrap(), U256::from_u64(255));
    assert_eq!(U256::from_hex("0x").unwrap_err(), u256::ParseError::Empty);
    assert!(U256::from_hex("0xzz").is_err());
}

#[test]
fn parses_one_ether_in_wei() {
    let one_eth = U256::from_hex("0xde0b6b3a7640000").unwrap();
    assert_eq!(one_eth.to_decimal(), "1000000000000000000");
    assert_eq!(one_eth.to_hex(), "0xde0b6b3a7640000");
}

#[test]
fn round_trips_a_full_width_word() {
    let hex = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    let v = U256::from_hex(hex).unwrap();
    assert_eq!(v, U256::MAX);
    assert_eq!(v.to_hex(), hex);
}

#[test]
fn rejects_more_than_32_bytes() {
    let too_long = format!("0x{}", "1".repeat(65));
    assert_eq!(
        U256::from_hex(&too_long).unwrap_err(),
        u256::ParseError::TooLong
    );
}

#[test]
fn addition_carries_across_limbs() {
    let a = U256::from_hex("0xffffffffffffffff").unwrap();
    let b = U256::from_u64(1);
    let sum = a.checked_add(b).unwrap();
    assert_eq!(sum.to_hex(), "0x10000000000000000");
}

#[test]
fn addition_overflow_is_refused() {
    assert!(U256::MAX.checked_add(U256::from_u64(1)).is_none());
}

#[test]
fn subtraction_borrows_across_limbs() {
    let a = U256::from_hex("0x10000000000000000").unwrap();
    let b = U256::from_u64(1);
    assert_eq!(a.checked_sub(b).unwrap().to_hex(), "0xffffffffffffffff");
}

#[test]
fn subtraction_underflow_is_refused() {
    assert!(U256::ZERO.checked_sub(U256::from_u64(1)).is_none());
}

#[test]
fn multiply_and_divide_by_small_integers() {
    let v = U256::from_u64(1_000_000_000);
    let scaled = v.checked_mul_u64(1_000_000_000).unwrap();
    assert_eq!(scaled.to_decimal(), "1000000000000000000");
    assert_eq!(scaled.checked_div_u64(1_000_000_000).unwrap(), v);
    assert!(v.checked_div_u64(0).is_none());
}

#[test]
fn decimal_output_matches_known_weis() {
    assert_eq!(
        U256::from_hex("0x38d7ea4c68000").unwrap().to_decimal(),
        "1000000000000000"
    );
    assert_eq!(U256::ZERO.to_decimal(), "0");
    assert_eq!(U256::from_u64(42).to_decimal(), "42");
}

#[test]
fn ordering_is_numeric_not_lexicographic() {
    assert!(U256::from_u64(2) < U256::from_u64(10));
    assert!(U256::from_hex("0x0100").unwrap() > U256::from_hex("0xff").unwrap());
}
