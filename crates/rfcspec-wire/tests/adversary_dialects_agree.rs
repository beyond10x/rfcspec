use rfcspec_abnf::Dialect;
use rfcspec_model::wire::AbnfDialect;

#[test]
fn abnf_dialect_and_model_dialect_share_every_wire_name() {
    let model = [
        AbnfDialect::Rfc5234,
        AbnfDialect::Rfc2616,
        AbnfDialect::Bits,
    ];
    assert_eq!(model.len(), Dialect::ALL.len());
    for (dialect, abnf) in model.into_iter().zip(Dialect::ALL) {
        let name = rfcspec_model::wire(&dialect);
        assert_eq!(name, abnf.wire());
        assert_eq!(Dialect::from_wire(&name), Some(abnf));
    }
}
