use std::{
    fmt::Debug,
    io::Write,
    process::{Command, Stdio},
};

use proc_macro2::TokenStream;

//
// Taken from https://github.com/rust-lang/rustfmt/issues/3257#issuecomment-523573838.
//
pub fn format_rust_expression(value: String) -> String {
    const PREFIX: &str = "";
    const SUFFIX: &str = "\n";

    if let Ok(mut proc) = Command::new("rustfmt")
        .arg("--emit=stdout")
        .arg("--edition=2018")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(PREFIX.as_bytes()).unwrap();
            stdin.write_all(value.trim().as_bytes()).unwrap();
            stdin.write_all(SUFFIX.as_bytes()).unwrap();
        };
        if let Ok(output) = proc.wait_with_output() {
            if output.status.success() {
                // slice between after the prefix and before the suffix
                // (currently 14 from the start and 2 before the end, respectively)
                let start = PREFIX.len() + 1;
                let end = output.stdout.len() - SUFFIX.len();
                return std::str::from_utf8(&output.stdout[start..end])
                    .unwrap()
                    .to_owned();
            } else {
                panic!("not a rust code 3 {}", value);
            }
        } else {
            panic!("not a rust code 2");
        }
    } else {
        panic!("not a rust code");
    }
}

pub fn test_code(input: TokenStream, target: TokenStream) {
    pretty_assertions::assert_eq!(
        format_rust_expression(input.to_string()),
        format_rust_expression(target.to_string())
    );
}

pub fn test_structs<T: Debug>(input: T, target: T) {
    pretty_assertions::assert_eq!(format!("{:#?}", input), format!("{:#?}", target));
}
