//! A fractional number must survive serialization in either precision mode.
//!
//! `rust_decimal`'s `to_u64`/`to_i64` truncate instead of returning `None` for a value with a
//! fractional part, so trying them before `to_f64` silently turned 44.12 into 44.

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use zen_types::variable::Variable;

fn json(v: &Variable) -> String {
    serde_json::to_string(v).expect("serialize")
}

#[test]
fn fractional_numbers_keep_their_fractional_part() {
    for raw in [44.12f64, 0.35, 168.4, -7.5, 0.0095] {
        let v = Variable::Number(Decimal::from_f64(raw).unwrap());
        let out = json(&v);
        let parsed: f64 = out.parse().unwrap_or_else(|_| {
            // arbitrary_precision wraps the number in a token struct; pull the digits back out.
            let s: serde_json::Value = serde_json::from_str(&out).unwrap();
            s.as_str()
                .map(str::to_owned)
                .or_else(|| {
                    s.get("$serde_json::private::Number")
                        .and_then(|n| n.as_str().map(str::to_owned))
                })
                .unwrap_or(out.clone())
                .parse()
                .unwrap()
        });
        assert!(
            (parsed - raw).abs() < 1e-9,
            "{raw} serialized as {out}, which parsed back as {parsed}"
        );
    }
}

#[test]
fn whole_numbers_still_serialize_without_a_decimal_point() {
    let v = Variable::Number(Decimal::from(80));
    assert_eq!(json(&v), "80");
}

#[test]
fn negative_whole_numbers_round_trip() {
    let v = Variable::Number(Decimal::from(-7));
    assert_eq!(json(&v), "-7");
}
