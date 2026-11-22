use picojson_safe_rust::{parse, parse_with_depth, parse_with_rest, Array, Object, Value};

#[test]
fn test_constructors_and_serialization() {
    assert_eq!(Value::from(true).serialize(), "true");
    assert_eq!(Value::from(false).serialize(), "false");
    assert_eq!(Value::from(42.0).serialize(), "42");
    assert_eq!(Value::from("hello").serialize(), "\"hello\"");
    assert_eq!(Value::from(3.0).serialize(), "3");
}

#[test]
fn test_primitives_parsing() {
    let v = parse("false").expect("false parse");
    assert_eq!(v, Value::Bool(false));
    assert_eq!(v.as_bool(), Some(false));
    assert_eq!(v.serialize(), "false");

    let v = parse("true").expect("true parse");
    assert_eq!(v, Value::Bool(true));
    assert_eq!(v.as_bool(), Some(true));
    assert_eq!(v.serialize(), "true");

    let v = parse("null").expect("null parse");
    assert_eq!(v, Value::Null);
    assert!(v.is_null());
    assert_eq!(v.serialize(), "null");

    let v = parse("90.5").expect("90.5 parse");
    assert!(v.is_number());
    assert_eq!(v.as_f64(), Some(90.5));

    let v = parse("1.7976931348623157e+308").expect("DBL_MAX parse");
    assert!(v.is_number());
    assert_eq!(v.as_f64(), Some(f64::MAX));

    let v = parse("\"hello\"").expect("string parse");
    assert_eq!(v.as_str(), Some("hello"));
    assert_eq!(v.serialize(), "\"hello\"");

    let v = parse(r#""\"\\/\b\f\n\r\t""#).expect("escapes parse");
    assert_eq!(v.as_str(), Some("\"\\/\x08\x0c\n\r\t"));
    assert_eq!(v.serialize(), r#""\"\\/\b\f\n\r\t""#);

    // Unicode \u0061\u30af\u30ea\u30b9 -> aクリス
    let v = parse(r#""\u0061\u30af\u30ea\u30b9""#).expect("unicode parse");
    assert_eq!(v.as_str(), Some("aクリス"));

    // Surrogate pair \ud840\udc0b -> \u{2000b}
    let v = parse(r#""\ud840\udc0b""#).expect("surrogate pair parse");
    assert_eq!(v.as_str(), Some("\u{2000b}"));
}

#[test]
fn test_int64_support() {
    let v = parse("0").expect("0 parse");
    assert_eq!(v.as_i64(), Some(0));
    assert_eq!(v.as_f64(), Some(0.0));
    assert_eq!(v.serialize(), "0");

    let v = parse("-9223372036854775808").expect("i64 min parse");
    assert_eq!(v.as_i64(), Some(i64::MIN));
    assert_eq!(v.serialize(), "-9223372036854775808");

    let v = parse("9223372036854775807").expect("i64 max parse");
    assert_eq!(v.as_i64(), Some(i64::MAX));
    assert_eq!(v.serialize(), "9223372036854775807");

    // Underflowing i64 becomes float
    let v = parse("-9223372036854775809").expect("underflow i64 parse");
    assert!(!v.is_int64());
    assert!(v.is_number());
    assert!((v.as_f64().unwrap() + 9.22337203685478e+18).abs() < 65536.0);
}

#[test]
fn test_empty_containers() {
    let v = parse("[]").expect("empty array");
    assert!(v.is_array());
    assert_eq!(v.as_array().unwrap().len(), 0);

    let v = parse("{}").expect("empty object");
    assert!(v.is_object());
    assert_eq!(v.as_object().unwrap().len(), 0);
}

#[test]
fn test_array_operations() {
    let s = "[1, true, \"hello\"]";
    let v = parse(s).expect("array parse");
    assert!(v.is_array());
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert!(v.contains_index(0));
    assert_eq!(v.get_index(0).and_then(|x| x.as_f64()), Some(1.0));
    assert!(v.contains_index(1));
    assert_eq!(v.get_index(1).and_then(|x| x.as_bool()), Some(true));
    assert!(v.contains_index(2));
    assert_eq!(v.get_index(2).and_then(|x| x.as_str()), Some("hello"));
    assert!(!v.contains_index(3));
}

#[test]
fn test_object_operations() {
    let s = r#"{ "a": true }"#;
    let v = parse(s).expect("object parse");
    assert!(v.is_object());
    let obj = v.as_object().unwrap();
    assert_eq!(obj.len(), 1);
    assert!(v.contains_key("a"));
    assert_eq!(v.get_key("a").and_then(|x| x.as_bool()), Some(true));
    assert_eq!(v.serialize(), r#"{"a":true}"#);
    assert!(!v.contains_key("z"));
}

#[test]
fn test_object_mutation_and_serialization() {
    let mut v1 = Value::Object(Object::new());
    if let Some(obj) = v1.as_object_mut() {
        obj.insert("114".to_string(), Value::from("514"));

        let mut sub_arr = Array::new();
        sub_arr.push(Value::from(334.0));
        obj.insert("364".to_string(), Value::Array(sub_arr));

        let mut sub_obj = Object::new();
        sub_obj.insert("893".to_string(), Value::from(810.0));
        obj.insert("1919".to_string(), Value::Object(sub_obj));
    }

    assert_eq!(
        v1.serialize(),
        r#"{"114":"514","1919":{"893":810},"364":[334]}"#
    );
}

#[test]
fn test_syntax_errors() {
    assert!(parse("falsoa").is_err());
    assert!(parse("{]").is_err());
    assert!(parse("\n\x08bell").is_err());
    assert!(parse("\"abc\nd\"").is_err());
}

#[test]
fn test_deep_equality() {
    let s1 = r#"{ "b": true, "a": [1,2,"three"], "d": 2 }"#;
    let s2 = r#"{ "d": 2, "b": true, "a": [1,2,"three"] }"#;
    let v1 = parse(s1).expect("v1");
    let v2 = parse(s2).expect("v2");
    assert_eq!(v1, v2);

    let s3 = r#"{ "d": 2, "a": [1,"three"], "b": true }"#;
    let v3 = parse(s3).expect("v3");
    assert_ne!(v1, v3);

    let s4 = r#"{ "d": 2, "a": [1,2,"three"], "b": false }"#;
    let v4 = parse(s4).expect("v4");
    assert_ne!(v1, v4);
}

#[test]
fn test_pretty_printing() {
    let s = r#"{ "a": 1, "b": [ 2, { "b1": "abc" } ], "c": {}, "d": [] }"#;
    let v = parse(s).expect("parse complex");
    assert_eq!(
        v.serialize(),
        r#"{"a":1,"b":[2,{"b1":"abc"}],"c":{},"d":[]}"#
    );
    let pretty = v.serialize_pretty();
    let expected = "{\n  \"a\": 1,\n  \"b\": [\n    2,\n    {\n      \"b1\": \"abc\"\n    }\n  ],\n  \"c\": {},\n  \"d\": []\n}\n";
    assert_eq!(pretty, expected);
}

#[test]
fn test_nan_and_inf_rejection() {
    let r = std::panic::catch_unwind(|| {
        let _ = Value::from(f64::NAN);
    });
    assert!(r.is_err(), "Should panic on NaN");

    let r = std::panic::catch_unwind(|| {
        let _ = Value::from(f64::INFINITY);
    });
    assert!(r.is_err(), "Should panic on Infinity");
}

#[test]
fn test_evaluate_as_boolean() {
    assert!(!Value::Null.evaluate_as_boolean());
    assert!(!Value::from(false).evaluate_as_boolean());
    assert!(Value::from(true).evaluate_as_boolean());
    assert!(!Value::from(0.0).evaluate_as_boolean());
    assert!(Value::from(1.0).evaluate_as_boolean());
    assert!(!Value::from(0i64).evaluate_as_boolean());
    assert!(Value::from(1i64).evaluate_as_boolean());
    assert!(!Value::from("").evaluate_as_boolean());
    assert!(Value::from("hello").evaluate_as_boolean());
    assert!(!Value::Array(vec![]).evaluate_as_boolean());
    assert!(Value::Array(vec![Value::from(1)]).evaluate_as_boolean());
    assert!(!Value::Object(Object::new()).evaluate_as_boolean());
}

#[test]
fn test_parse_with_rest() {
    let (v, rest) = parse_with_rest("123abc", 512).expect("parse_with_rest");
    assert_eq!(v.as_i64(), Some(123));
    assert_eq!(rest, "abc");
}

#[test]
fn test_max_depth() {
    let s = "[[[[[[[[[[ 1 ]]]]]]]]]]";
    assert!(parse_with_depth(s, 5).is_err());
    assert!(parse_with_depth(s, 20).is_ok());
}

#[test]
fn test_powers_of_two_roundtrip() {
    let mut a: f64 = 1.0;
    for i in 0..1024 {
        let vi = Value::from(a);
        let serialized = vi.serialize();
        let vo = parse(&serialized).expect("roundtrip parse");
        let b = vo.as_f64().expect("as_f64");
        if i < 53 {
            assert_eq!(a, b, "Exact match expected for i = {}", i);
        } else {
            let rel_diff = (a - b).abs() / b;
            assert!(rel_diff <= 1e-8, "Relative difference too large at i = {}: a={}, b={}", i, a, b);
        }
        a *= 2.0;
    }
}

#[test]
fn test_swap_operations() {
    let mut v1 = Value::from(true);
    let mut v2 = Value::Null;
    std::mem::swap(&mut v1, &mut v2);
    assert!(v1.is_null());
    assert_eq!(v2.as_bool(), Some(true));

    let mut v1 = Value::from("a");
    let mut v2 = Value::from(1.0);
    std::mem::swap(&mut v1, &mut v2);
    assert_eq!(v1.as_f64(), Some(1.0));
    assert_eq!(v2.as_str(), Some("a"));

    let mut v1 = Value::Object(Object::new());
    let mut v2 = Value::Array(Array::new());
    std::mem::swap(&mut v1, &mut v2);
    assert!(v1.is_array());
    assert!(v2.is_object());
}

#[test]
fn test_erase_and_modify() {
    let s = r#"{ "b": true, "a": [1,2,"three"], "d": 2 }"#;
    let mut v1 = parse(s).expect("v1 parse");
    if let Some(obj) = v1.as_object_mut() {
        obj.remove("b");
        if let Some(arr) = obj.get_mut("a").and_then(|v| v.as_array_mut()) {
            arr.retain(|x| x.as_str() != Some("three"));
        }
    }
    let s2 = r#"{ "a": [1,2], "d": 2 }"#;
    let v2 = parse(s2).expect("v2 parse");
    assert_eq!(v1, v2);
}

