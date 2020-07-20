pub fn pretty_print(request: Vec<u8>) -> String {
    let vec: Vec<u8> = request;
    format!(
        r#"
########################################################
{}
########################################################
"#,
        String::from_utf8(vec).expect("bytes to string")
    )
}
