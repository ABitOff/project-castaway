#![allow(unreachable_code, dead_code)]

use project_beacon_macro_args_macro::parseable;

enum TestEnum {
    A,
    B,
    C,
    D,
}

parseable! {
    Test {
        foo: String,
        bar: ByteString,
        baz: Test1,
        qux: enum TestEnum[A, B, C, D],
        quux: map,
        foo_opt?: String,
        bar_opt?: ByteString,
        baz_opt?: Test1,
        qux_opt?: enum TestEnum[A, B, C, D],
        quux_opt?: map,
        foo_def: String; "123abc",
        bar_def: ByteString; b"\x01\x02\x03\x0a\x0b\x0c",
        qux_def: enum TestEnum[A, B, C, D; C],
    }
    Test1 {
        a: String,
        b: ByteString,
        c: map,
    }
}

mod std {
    pub fn std_mod() {}
}
mod syn {
    pub fn syn_mod() {}
}
mod proc_macro {
    pub fn proc_macro_mod() {}
}
mod proc_macro2 {
    pub fn proc_macro2_mod() {}
}
mod quote {
    pub fn quote_mod() {}
}

#[test]
fn test_simple() {
    let _ = Test {
        foo: "Abc".to_string(),
        bar: b"123",
        baz: Test1 {
            a: "123".to_string(),
            b: b"abc",
            c: ::std::collections::BTreeMap::<String, String>::new(),
        },
        qux: TestEnum::A,
        quux: ::std::collections::BTreeMap::<String, String>::new(),
        foo_opt: None,
        bar_opt: None,
        baz_opt: None,
        qux_opt: None,
        quux_opt: None,
        foo_def: "123abc".to_string(),
        bar_def: b"\x01\x02\x03\x0a\x0b\x0c",
        qux_def: TestEnum::C,
    };
}
