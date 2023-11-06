use std::error::Error;
use std::fs;
use std::path::PathBuf;

use crate::command_csv::command_csv_format;

#[test]
fn csv_test() -> Result<(), Box<dyn Error>> {
    let mut test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_file.push("resources/test/addresses.csv");

    let csv = fs::read_to_string(test_file)?;
    let mut out = Vec::new();
    command_csv_format(&csv, &mut out)?;

    let s = String::from_utf8(out)?;
    println!("{}", s);
    assert_eq!(s.lines().count(), 19);

    Ok(())
}

#[test]
fn csv_test_empty() -> Result<(), Box<dyn Error>> {
    let mut out = Vec::new();
    command_csv_format("", &mut out)?;

    let s = String::from_utf8(out)?;
    assert_eq!(s.len(), 0);

    Ok(())
}

#[test]
fn csv_test_headers_only() -> Result<(), Box<dyn Error>> {
    let mut out = Vec::new();
    command_csv_format("First Name, Last Name, Age", &mut out)?;

    let s = String::from_utf8(out)?;
    println!("{}", s);
    assert_eq!(s.lines().count(), 3);

    Ok(())
}

#[test]
fn csv_test_bigger_row() -> Result<(), Box<dyn Error>> {
    let mut out = Vec::new();
    command_csv_format(
        "
    First Name, Last Name, Age
    My Name, My Last Name, My Age, Something Extra
    ",
        &mut out,
    )?;

    let s = String::from_utf8(out)?;
    println!("{}", s);
    assert_eq!(s.lines().count(), 5);

    Ok(())
}
