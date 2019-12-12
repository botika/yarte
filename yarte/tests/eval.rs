use yarte::Template;

#[derive(Template)]
#[template(src = "{{ \"foo\" + \"bar\" }}")]
struct AddStrTemplate;

#[test]
fn test_add_str() {
    let t = AddStrTemplate;
    assert_eq!(t.call().unwrap(), "foobar");
}

#[derive(Template)]
#[template(src = "{{ \"foo\" + \"bar\" * 2 }}")]
struct MulStrTemplate;

#[test]
fn test_mul_str() {
    let t = MulStrTemplate;
    assert_eq!(t.call().unwrap(), "foobarbar");
}

#[derive(Template)]
#[template(src = "{{ (\"foo\" + \"bar\") * 2 }}")]
struct MulParenStrTemplate;

#[test]
fn test_mul_paren_str() {
    let t = MulParenStrTemplate;
    assert_eq!(t.call().unwrap(), "foobarfoobar");
}

#[derive(Template)]
#[template(
    src = "
    {{~# each &[\"foo\", \"bar\"] ~}}
        {{ this }} {{ index }}
    {{~/ each }}
    ",
    print = "code"
)]
struct ConstForTemplate;

#[test]
fn test_const_for() {
    let t = ConstForTemplate;
    assert_eq!(t.call().unwrap(), "foo 1bar 2");
}

#[derive(Template)]
#[template(
    src = "
    {{~# each 1..10 ~}}
        {{ this }} {{ index }}
    {{~/ each }}
    ",
    print = "code"
)]
struct ConstForRangeTemplate;

#[test]
fn test_const_for_range() {
    let t = ConstForRangeTemplate;
    assert_eq!(t.call().unwrap(), "1 12 23 34 45 56 67 78 89 9");
}

#[derive(Template)]
#[template(
    src = "
    {{~# each &[\"foo\", \"bar\"] ~}}
        {{# each 2..4 ~}}
            {{ super::index }} {{ this }}
        {{~/each }} {{ this }}
    {{~/ each }}
    ",
    print = "code"
)]
struct ConstForNestedTemplate;

#[test]
fn test_const_for_nested() {
    let t = ConstForNestedTemplate;
    assert_eq!(t.call().unwrap(), "1 21 3 foo2 22 3 bar");
}

#[derive(Template)]
#[template(
    src = "
    {{~# each &[\"foo\", \"bar\"] ~}}
        {{# each 2..4 ~}}
            {{ super::index }} {{ super::super::_0 }} {{ this }}
        {{~/each }} {{ this }}
    {{~/ each }}
    ",
    print = "code"
)]
struct ConstForNested2Template<'a>(&'a str);

#[test]
fn test_const_for_nested2() {
    let t = ConstForNested2Template("fol");
    assert_eq!(t.call().unwrap(), "1 fol 21 fol 3 foo2 fol 22 fol 3 bar");
}

#[derive(Template)]
#[template(path = "eval-partial.hbs", print = "code")]
struct ConstPartialTemplate;

#[test]
fn test_const_partial() {
    let t = ConstPartialTemplate;
    assert_eq!(
        t.call().unwrap(),
        "foo\npartial in subdirectory\n1bar\npartial in subdirectory\n2"
    );
}

#[derive(Template)]
#[template(path = "eval-partial-range.hbs", print = "code")]
struct ConstPartialRangeTemplate;

#[test]
fn test_const_partial_range() {
    let t = ConstPartialRangeTemplate;
    assert_eq!(
        t.call().unwrap(),
        "0\npartial in subdirectory\n11\npartial in subdirectory\n2"
    );
}

#[derive(Template)]
#[template(path = "eval-partial-str.hbs", print = "code")]
struct ConstPartialStrTemplate;

#[test]
fn test_const_partial_str() {
    let t = ConstPartialStrTemplate;
    assert_eq!(
        t.call().unwrap(),
        "f\npartial in subdirectory\n1o\npartial in subdirectory\n2o\npartial in subdirectory\n3"
    );
}
