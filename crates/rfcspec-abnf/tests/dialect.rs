use rfcspec_abnf::Dialect;

#[test]
fn dialect_reads_and_writes_each_wire_name() {
    for (wire, dialect) in [
        ("rfc5234", Dialect::Rfc5234),
        ("rfc2616", Dialect::Rfc2616),
        ("bits", Dialect::Bits),
    ] {
        assert_eq!(Dialect::from_wire(wire), Some(dialect), "{wire}");
        assert_eq!(dialect.wire(), wire);
    }
}

#[test]
fn dialect_refuses_a_name_outside_the_three() {
    for wire in ["abnf", "", "RFC5234", "Bits", "rfc7405"] {
        assert_eq!(Dialect::from_wire(wire), None, "{wire}");
    }
}
