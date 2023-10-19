use std::cell::RefCell;
use std::cmp::max;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Write;

pub(super) fn command_csv_format(text: &str, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    let mut csv = CSVTable::default();

    // read csv
    // reader has support for quotes, multi lines etc.
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    // set header row
    csv.set_header(rdr.headers()?.iter().map(|h| h.trim().to_owned()).collect());

    // set data rows
    for row in rdr.records() {
        csv.append(row?.iter().map(|h| h.trim().to_owned()).collect());
    }

    // invoke .to_string() on csv table
    write!(out, "{}", csv)?;

    Ok(())
}

// Holds CSV table structure
#[derive(Default)]
struct CSVTable {
    headers: Vec<Vec<String>>,   // columns with multiple lines
    rows: Vec<Vec<Vec<String>>>, // list of columns with multiple lines
    column_sizes: Vec<usize>,    // max size of a column
}

// cell padding of text
// note as this is constant, setting PADDING = 1 will produce
// clippy warnings that "".repeat(1) should be removed :-).
const PADDING: usize = 2;

impl CSVTable {
    // Sets csv header.
    fn set_header(&mut self, headers: Vec<String>) {
        // set headers
        self.headers = CSVTable::parse_row(headers);

        // set initial column width
        for header in self.headers.iter() {
            self.column_sizes
                .push(header.iter().map(|h| h.len()).max().unwrap_or_default());
        }
    }

    // Append data to table.
    fn append(&mut self, row: Vec<String>) {
        // skip empty records
        if row.join("").is_empty() {
            return;
        }

        let mut row = row;

        // fix rows with less columns than headers.
        while row.len() < self.headers.len() {
            row.push("".to_owned());
        }

        // limit size of row to size of header
        row = row[0..self.headers.len()].into();

        // convert row to multi line columns
        let row = CSVTable::parse_row(row);

        // recompute max column width for index i
        for (i, column) in row.iter().enumerate() {
            let column_size = column.iter().map(|c| c.len()).max().unwrap_or_default();
            self.column_sizes[i] = max(self.column_sizes[i], column_size);
        }

        self.rows.push(row);
    }

    // Returns row with multi-line columns.
    fn parse_row(row: Vec<String>) -> Vec<Vec<String>> {
        let mut row_with_lines = Vec::new();

        for column in row {
            let lines: Vec<String> = column.lines().map(|l| l.to_owned()).collect();
            row_with_lines.push(lines);
        }

        row_with_lines
    }
}

impl Display for CSVTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.headers.is_empty() {
            // empty table
            return Ok(());
        }

        // needed for both closures to access f
        let f = RefCell::new(f);

        // write separator line
        let write_line = |line_number: usize, is_last_row: bool| -> std::fmt::Result {
            let mut f = f.borrow_mut();

            // define different characters for border based on line number
            let (left_corner, middle, middle_border, right_corner) = match line_number {
                0 => ("┏", "━", "┯", "┓"),                // top header
                1 if is_last_row => ("┗", "━", "┷", "┛"), // last row if there is only header
                _ if is_last_row => ("└", "─", "┴", "┘"), // last row
                1 => ("┡", "━", "┿", "┩"),                // 2nd line if there are more data
                _ => ("├", "─", "┼", "┤"),                // middle line
            };

            write!(f, "{}", left_corner)?;
            for (i, c) in self.column_sizes.iter().enumerate() {
                write!(f, "{}", middle.repeat(*c + PADDING * 2))?;

                // if not last column
                if i < self.column_sizes.len() - 1 {
                    write!(f, "{}", middle_border)?;
                }
            }
            writeln!(f, "{}", right_corner)?;

            Ok(())
        };

        // write data line
        let write_data =
            |line_number: usize, row: &[Vec<String>], column_number: usize| -> std::fmt::Result {
                let mut f = f.borrow_mut();

                // define thick border line for header row
                let (border, middle) = match line_number {
                    1 => ("┃", "│"), // header row
                    _ => ("│", "│"),
                };

                write!(f, "{}", border)?;
                for (i, column) in row.iter().enumerate() {
                    let data = if column_number < column.len() {
                        &column[column_number]
                    } else {
                        ""
                    };

                    // write text data surrounded by padding and alignment
                    write!(
                        f,
                        "{}{}{}",
                        " ".repeat(PADDING),
                        data,
                        " ".repeat(PADDING + self.column_sizes[i] - data.len())
                    )?;

                    // if not last column
                    if i < self.column_sizes.len() - 1 {
                        write!(f, "{}", middle)?;
                    }
                }

                // right border
                writeln!(f, "{}", border)?;

                Ok(())
            };

        write_line(0, false)?;

        for (i, row) in [self.headers.clone()]
            .iter()
            .chain(self.rows.iter())
            .enumerate()
        {
            for column_number in 0..row
                .iter()
                .map(|column| column.len())
                .max()
                .unwrap_or_default()
            {
                write_data(i + 1, row, column_number)?;
            }

            write_line(i + 1, i >= self.rows.len())?;
        }

        Ok(())
    }
}
