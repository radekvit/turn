#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("test/test01-parse.rs");
    t.compile_fail("test/test02-reject_struct.rs");
    t.compile_fail("test/test03-reject_union.rs");
    t.compile_fail("test/test04-reject_data_enum.rs");
    t.compile_fail("test/test05-reject_explicit_discriminant.rs");
    t.compile_fail("test/test06-reject_global_attrs.rs");
    t.compile_fail("test/test07-reject_invalid_skip.rs");
    t.compile_fail("test/test08-reject_multiple_skip.rs");
    t.compile_fail("test/test09-reject_invalid_regex.rs");
}
