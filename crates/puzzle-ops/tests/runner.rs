use trybuild::TestCases;

#[test]
fn runner() {
    let t = TestCases::new();
    t.pass("tests/pass/*.rs");
}
