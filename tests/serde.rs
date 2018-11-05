extern crate serde_json;
extern crate small;

#[cfg(all(feature = "serde", feature = "std"))]
#[test]
fn string_to_json_interop() {
    assert_eq!(
        std::string::String::from("\"testing a string\""),
        serde_json::to_string(&String::from("testing a string")).unwrap()
    );

    let x = small::String::from("hello, how are you?");
    let y: small::String = serde_json::from_str(&serde_json::to_string(&x).unwrap()).unwrap();

    assert_eq!(x, y);
}
