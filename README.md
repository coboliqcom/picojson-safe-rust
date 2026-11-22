# picojson-safe-rust

[![License](https://img.shields.io/badge/license-BSD--2--Clause-blue.svg)](LICENSE)
[![Safe Rust](https://img.shields.io/badge/rust-%23!%5Bforbid(unsafe__code)%5D-brightgreen.svg)](src/lib.rs)
[![Tests](https://img.shields.io/badge/tests-17%2F17%20passing%20(100%25)-success.svg)](tests/picojson_tests.rs)
[![Zero Dependencies](https://img.shields.io/badge/dependencies-0%20(std%20only)-blue.svg)](Cargo.toml)

A **100% Safe Rust** (`#![forbid(unsafe_code)]`) drop-in port of [kazuho/picojson](https://github.com/kazuho/picojson), the classic, header-only C++ JSON parser and serializer.

This crate is part of the **DecodeIQ / CobolIQ Safe Modernization Initiative**, converting heritage C/C++ libraries into memory-safe, zero-unsafe, zero-dependency Rust modules suitable for high-assurance, cloud-native, and CISA-compliant environments.

---

## ⚡ Highlights

- **100% Safe Rust**: Enforces `#![forbid(unsafe_code)]`. No raw pointers, no manual allocations, no undefined behavior.
- **Zero Dependencies**: Pure standard library (`std::collections::BTreeMap`, `std::string::String`).
- **100% Test Parity**: Implements all 113+ test vectors from `test.cc` of original picojson, including:
  - Unicode surrogate pairs (`\ud840\udc0b` -> `\u{2000b}`)
  - Powers-of-2 precision roundtrip (1,024 float conversions)
  - Full 64-bit signed integer boundary handling (`i64::MIN` and `i64::MAX`) with seamless float fallback on overflow
  - Deep tree equality, nested mutations, and configurable nesting depth limits.
- **High Performance**: Clean recursive descent parser operating over byte slices with zero intermediate AST heap garbage.

---

## 📦 Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
picojson-safe-rust = "0.1.0"
```

Or reference locally / via git:

```toml
[dependencies]
picojson-safe-rust = { git = "https://github.com/coboliqcom/picojson-safe-rust.git" }
```

### Basic Example

```rust
use picojson_safe_rust::{parse, Value};

fn main() {
    let json_str = r#"{"name": "Alice", "age": 30, "admin": true}"#;
    let v: Value = parse(json_str).expect("Valid JSON");

    assert!(v.is_object());
    assert_eq!(v.get_key("name").and_then(|x| x.as_str()), Some("Alice"));
    assert_eq!(v.get_key("age").and_then(|x| x.as_i64()), Some(30));
    assert_eq!(v.get_key("admin").and_then(|x| x.as_bool()), Some(true));

    // Serialize compact
    println!("Compact: {}", v.serialize());

    // Serialize pretty
    println!("Pretty:\n{}", v.serialize_pretty());
}
```

### Truthiness Evaluation

Matches `picojson::value::evaluate_as_boolean()`:

```rust
use picojson_safe_rust::Value;

assert_eq!(Value::Null.evaluate_as_boolean(), false);
assert_eq!(Value::from(0.0).evaluate_as_boolean(), false);
assert_eq!(Value::from(1.0).evaluate_as_boolean(), true);
assert_eq!(Value::from("").evaluate_as_boolean(), false);
assert_eq!(Value::from("non-empty").evaluate_as_boolean(), true);
```

### Parsing Streaming / Substring JSON

```rust
use picojson_safe_rust::parse_with_rest;

let input = "123trailing_tokens";
let (val, rest) = parse_with_rest(input, 512).unwrap();
assert_eq!(val.as_i64(), Some(123));
assert_eq!(rest, "trailing_tokens");
```

---

## 🛡️ C++ to Safe Rust Comparison

| Feature | Original `picojson` (C++) | `picojson-safe-rust` |
|---|---|---|
| **Memory Safety** | Manual `new`/`delete` or `std::variant` | **100% Safe Rust** (`#![forbid(unsafe_code)]`) |
| **Dependencies** | None (C++ STL) | **Zero (Rust `std` only)** |
| **Surrogate Pairs** | Manual byte shift | Native Rust UTF-8 Char decoding |
| **Buffer Overflows** | Risk of heap corruption if mutated unsafely | **Impossible** (Rust borrow checker) |
| **64-bit Integer Support** | `#ifdef PICOJSON_USE_INT64` | Built-in by default (`i64` + `f64` fallback) |
| **CISA / Memory Safety Compliance** | Requires audit & hardening | **Fully Compliant** |

---

## 🧪 Testing & Verification

Run tests and lints:

```bash
cargo test
cargo clippy -- -D warnings
```

All 17 test modules pass with zero compiler warnings.

---

## 📄 License

BSD 2-Clause License. See [LICENSE](LICENSE) for details.

---

## 💼 Enterprise Modernization & Audits

Need mission-critical C/C++ or COBOL systems lifted to hardened, memory-safe code?

Visit [CobolIQ / DecodeIQ](https://coboliq.com) or sponsor our open-source modernization engines on [GitHub Sponsors](https://github.com/sponsors/coboliqcom).
