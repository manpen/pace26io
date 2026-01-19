use crate::pace::parameters::tree_decomposition::TreeDecomposition;
use std::io::BufRead;
use thiserror::Error;

/// Reads an instance in the PACE 2026 format.
///
/// The reader is implemented using the Visitor pattern. It processes the input line by line,
/// invoking methods on the provided `InstanceVisitor` implementation for each recognized element.
///
/// To use the reader, implement the `InstanceVisitor` trait and create an `InstanceReader`
/// with a mutable reference to your visitor. Then call the `read` method with a type that
/// implements `BufRead`, such as a file or an in-memory buffer.
///
/// # Example
/// ```
/// use std::io::BufReader;
/// use pace26io::pace::reader::*;
///
/// struct MyVisitor {}
///
/// impl InstanceVisitor for MyVisitor {
///   fn visit_header(&mut self, lineno: usize, num_trees: usize, num_leaves: usize) -> Action {
///      println!("Header at line {}: {} trees, {} leaves", lineno+1, num_trees, num_leaves);
///      Action::Continue
///   }
///
///   fn visit_tree(&mut self, lineno: usize, line: &str) -> Action {
///      println!("Tree at line {}: {}", lineno + 1, line);
///      Action::Continue
///   }
/// }
///
/// let input = "#p 2 3\n(1);\n(2);";
/// let mut visitor = MyVisitor {};
/// let mut reader = InstanceReader::new(&mut visitor);
/// reader.read(BufReader::new(input.as_bytes())).unwrap();
/// ```
pub struct InstanceReader<'a, V: InstanceVisitor> {
    visitor: &'a mut V,
}

/// Visitor trait for processing elements of a PACE 2026 instance.
/// You need only implement the methods you care about, and ignore the rest.
/// A typical solver implementation would implement `visit_tree`
/// and possibly `visit_header`.
///
/// Example: see the documentation of [`InstanceReader`].
pub trait InstanceVisitor {
    fn visit_header(&mut self, _lineno: usize, _num_trees: usize, _num_leaves: usize) -> Action {
        Action::Continue
    }
    fn visit_approx_line(&mut self, _lineno: usize, _param_a: f64, _param_b: usize) -> Action {
        Action::Continue
    }

    fn visit_tree(&mut self, _lineno: usize, _line: &str) -> Action {
        Action::Continue
    }
    fn visit_line_with_extra_whitespace(&mut self, _lineno: usize, _line: &str) -> Action {
        Action::Continue
    }
    fn visit_unrecognized_hash_line(&mut self, _lineno: usize, _line: &str) -> Action {
        Action::Continue
    }
    fn visit_unrecognized_line(&mut self, _lineno: usize, _line: &str) -> Action {
        Action::Continue
    }
    fn visit_stride_line(
        &mut self,
        _lineno: usize,
        _line: &str,
        _key: &str,
        _value: &str,
    ) -> Action {
        Action::Continue
    }

    const VISIT_PARAM_TREE_DECOMPOSITION: bool = false;
    /// Is only called if `Self::VISIT_PARAM_TREE_DECOMPOSITION == true`.
    fn visit_param_tree_decomposition(&mut self, _lineno: usize, _td: TreeDecomposition) -> Action {
        Action::Continue
    }
}

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error("Identified line {} as header. Expected '#p {{numtree}} {{numleaves}}'", lineno+1)]
    InvalidHeaderLine { lineno: usize },

    #[error("Identified line {} as stride line. Expected '#s {{key}}: {{value}}'", lineno+1)]
    InvalidStrideLine { lineno: usize },

    #[error("Identified line {} as parameter line. Expected '#x {{key}}: {{value}}'", lineno+1)]
    InvalidParameterLine { lineno: usize },

    #[error("Identified line {} as approx line. Expected '#a {{a}} {{b}}'", lineno+1)]
    InvalidApproxLine { lineno: usize },

    #[error("Unknown parameter in line {}: {key}'", lineno+1)]
    UnknownParameter { lineno: usize, key: String },

    #[error("Invalid JSON in line {}: {err}", lineno + 1)]
    InvalidJSON {
        lineno: usize,
        err: serde_json::Error,
    },

    #[error("Found multiple headers. Lines {} and {}", lineno0+1, lineno1+1)]
    MultipleHeaders { lineno0: usize, lineno1: usize },

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

fn try_parse_header(line: &str) -> Option<(usize, usize)> {
    let mut parts = line.split(' ');
    if parts.next()? != "#p" {
        return None;
    }

    let num_trees = parts.next().and_then(|x| x.parse::<usize>().ok())?;
    let num_leaves = parts.next().and_then(|x| x.parse::<usize>().ok())?;

    Some((num_trees, num_leaves))
}

fn try_parse_approx(line: &str) -> Option<(f64, usize)> {
    let mut parts = line.split(' ');
    if parts.next()? != "#a" {
        return None;
    }

    let param_a = parts.next().and_then(|x| x.parse::<f64>().ok())?;
    if param_a < 0.0 {
        return None;
    }
    let param_b = parts.next().and_then(|x| x.parse::<usize>().ok())?;

    Some((param_a, param_b))
}

/// Expects a line `#X {key} {value}` and returns ({key}, {value}) if found
fn try_split_key_value(line: &str) -> Option<(&str, &str)> {
    let split = line[3..].find(' ')? + 3;

    let key = line[2..split].trim();
    let value = line[split + 1..].trim();

    Some((key, value))
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Continue,
    Terminate,
}

type ReaderResult<T> = std::result::Result<T, ReaderError>;

impl<'a, V: InstanceVisitor> InstanceReader<'a, V> {
    pub fn new(visitor: &'a mut V) -> Self {
        Self { visitor }
    }

    pub fn read<R: BufRead>(&mut self, reader: R) -> ReaderResult<()> {
        macro_rules! visit {
            ($method : ident, $( $args:expr ),* $(,)? ) => {
                if self.visitor.$method( $( $args ),*) == Action::Terminate
                {
                    return Ok(());
                }
            };
        }

        let mut header_line = None;
        for (lineno, line) in reader.lines().enumerate() {
            let line = line?;
            let content = line.trim();

            if content.len() != line.len() {
                // line has extra whitespace
                visit!(visit_line_with_extra_whitespace, lineno, &line);
            }

            // empty line
            if content.is_empty() {
                continue;
            }

            if content.starts_with("#") {
                if content.starts_with("# ") {
                    // comment, nothing to do
                } else if content.starts_with("#p") {
                    // header line

                    // make sure header is unique
                    if let Some(lineno0) = header_line {
                        return Err(ReaderError::MultipleHeaders {
                            lineno0,
                            lineno1: lineno,
                        });
                    } else {
                        header_line = Some(lineno);
                    }

                    if let Some((num_trees, num_leaves)) = try_parse_header(content) {
                        visit!(visit_header, lineno, num_trees, num_leaves);
                    } else {
                        return Err(ReaderError::InvalidHeaderLine { lineno });
                    }
                } else if content.starts_with("#s") {
                    // stride line in the format "#s key: value"
                    if let Some((key, value)) = try_split_key_value(content) {
                        visit!(visit_stride_line, lineno, content, key, value);
                    } else {
                        return Err(ReaderError::InvalidStrideLine { lineno });
                    }
                } else if content.starts_with("#a") {
                    // stride line in the format "#s key: value"
                    if let Some((a, b)) = try_parse_approx(content) {
                        visit!(visit_approx_line, lineno, a, b);
                    } else {
                        return Err(ReaderError::InvalidApproxLine { lineno });
                    }
                } else if content.starts_with("#x") {
                    if let Some((key, value)) = try_split_key_value(content) {
                        match key {
                            "treedecomp" => {
                                if V::VISIT_PARAM_TREE_DECOMPOSITION {
                                    match serde_json::from_str::<TreeDecomposition>(value) {
                                        Ok(td) => {
                                            visit!(visit_param_tree_decomposition, lineno, td);
                                        }
                                        Err(err) => {
                                            return Err(ReaderError::InvalidJSON { lineno, err });
                                        }
                                    };
                                }
                            }

                            _ => {
                                return Err(ReaderError::UnknownParameter {
                                    lineno,
                                    key: key.into(),
                                });
                            }
                        }
                    } else {
                        return Err(ReaderError::InvalidParameterLine { lineno });
                    }
                } else {
                    // unrecognized line
                    visit!(visit_unrecognized_hash_line, lineno, content);
                }
                continue;
            }

            if content.ends_with(";") {
                visit!(visit_tree, lineno, content);
                continue;
            }

            visit!(visit_unrecognized_line, lineno, content);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestVisitor {
        pub headers: Vec<(usize, usize, usize)>,
        pub trees: Vec<(usize, String)>,
        pub extra_whitespace_lines: Vec<(usize, String)>,
        pub unrecognized_hash_lines: Vec<(usize, String)>,
        pub unrecognized_lines: Vec<(usize, String)>,
        pub stride_lines: Vec<(usize, String, String, String)>,
        pub param_tree_decomp: Option<(usize, TreeDecomposition)>,
        pub approx_lines: Vec<(usize, f64, usize)>,
    }

    impl InstanceVisitor for TestVisitor {
        fn visit_header(&mut self, lineno: usize, num_trees: usize, num_leaves: usize) -> Action {
            self.headers.push((lineno, num_trees, num_leaves));
            Action::Continue
        }

        fn visit_tree(&mut self, lineno: usize, line: &str) -> Action {
            self.trees.push((lineno, line.to_string()));
            Action::Continue
        }

        fn visit_line_with_extra_whitespace(&mut self, lineno: usize, line: &str) -> Action {
            self.extra_whitespace_lines.push((lineno, line.to_string()));
            Action::Continue
        }

        fn visit_unrecognized_hash_line(&mut self, lineno: usize, line: &str) -> Action {
            self.unrecognized_hash_lines
                .push((lineno, line.to_string()));
            Action::Continue
        }

        fn visit_unrecognized_line(&mut self, lineno: usize, line: &str) -> Action {
            self.unrecognized_lines.push((lineno, line.to_string()));
            Action::Continue
        }

        fn visit_approx_line(&mut self, lineno: usize, param_a: f64, param_b: usize) -> Action {
            self.approx_lines.push((lineno, param_a, param_b));
            Action::Continue
        }

        fn visit_stride_line(
            &mut self,
            lineno: usize,
            line: &str,
            key: &str,
            value: &str,
        ) -> Action {
            self.stride_lines
                .push((lineno, line.to_string(), key.to_string(), value.to_string()));
            Action::Continue
        }

        const VISIT_PARAM_TREE_DECOMPOSITION: bool = true;
        fn visit_param_tree_decomposition(
            &mut self,
            lineno: usize,
            td: TreeDecomposition,
        ) -> Action {
            assert!(self.param_tree_decomp.is_none());
            self.param_tree_decomp = Some((lineno, td));
            Action::Continue
        }
    }

    #[test]
    fn test_valid_input() {
        let input = "#p 2 3\n(1);\n# comment\n(2);\n";

        let mut visitor = TestVisitor::default();
        let mut reader = InstanceReader::new(&mut visitor);
        reader.read(input.as_bytes()).unwrap();

        assert_eq!(visitor.headers, vec![(0, 2, 3)]);
        assert_eq!(
            visitor.trees,
            vec![(1, "(1);".to_string()), (3, "(2);".to_string())]
        );
        assert!(visitor.extra_whitespace_lines.is_empty());
        assert!(visitor.unrecognized_hash_lines.is_empty());
        assert!(visitor.unrecognized_lines.is_empty());
    }

    #[test]
    fn input_with_whitespace() {
        let input = "#p 2 3\n (1);\n\n(2);";

        let mut visitor = TestVisitor::default();
        let mut reader = InstanceReader::new(&mut visitor);
        reader.read(input.as_bytes()).unwrap();

        assert_eq!(visitor.headers, vec![(0, 2, 3)]);
        assert_eq!(
            visitor.trees,
            vec![(1, "(1);".to_string()), (3, "(2);".to_string())]
        );
        assert_eq!(
            visitor.extra_whitespace_lines,
            vec![(1, " (1);".to_string())]
        );
        assert!(visitor.unrecognized_hash_lines.is_empty());
        assert!(visitor.unrecognized_lines.is_empty());
    }

    #[test]
    fn input_with_unrecognized_lines() {
        let input = "#p 2 3\n (1);\n\n(2);\n#<illegal comment\n(3)missing semicolon";

        let mut visitor = TestVisitor::default();
        let mut reader = InstanceReader::new(&mut visitor);
        reader.read(input.as_bytes()).unwrap();

        assert_eq!(visitor.headers, vec![(0, 2, 3)]);
        assert_eq!(
            visitor.trees,
            vec![(1, "(1);".to_string()), (3, "(2);".to_string())]
        );
        assert_eq!(
            visitor.extra_whitespace_lines,
            vec![(1, " (1);".to_string())]
        );
        assert_eq!(
            visitor.unrecognized_hash_lines,
            vec![(4, "#<illegal comment".to_string())]
        );
        assert_eq!(
            visitor.unrecognized_lines,
            vec![(5, "(3)missing semicolon".to_string())]
        );
    }

    #[test]
    fn input_with_approx_line() {
        let input = "#p 2 3\n#s stride_key somevalue\n#a 1.2345 42\n(1);\n";
        let mut visitor = TestVisitor::default();
        let mut reader = InstanceReader::new(&mut visitor);
        reader.read(input.as_bytes()).unwrap();

        assert_eq!(visitor.approx_lines, vec![(2, 1.2345, 42)]);
    }

    #[test]
    fn input_with_invalid_approx_line() {
        for input in [
            "#p 2 3\n#s stride_key somevalue\n#a -1.2345 42\n(1);\n",
            "#a foo 3",
            "#a 1.234 foo",
            "#a 1.2345 -4",
        ] {
            let mut visitor = TestVisitor::default();
            let mut reader = InstanceReader::new(&mut visitor);
            let res = reader.read(input.as_bytes());
            assert!(
                matches!(res.unwrap_err(), ReaderError::InvalidApproxLine { .. }),
                "{input:?}"
            );
        }
    }

    #[test]
    fn input_with_stride_line() {
        let input = "#p 2 3\n#s stride_key somevalue\n(1);\n";
        let mut visitor = TestVisitor::default();
        let mut reader = InstanceReader::new(&mut visitor);
        reader.read(input.as_bytes()).unwrap();

        assert_eq!(
            visitor.stride_lines,
            vec![(
                1,
                "#s stride_key somevalue".to_string(),
                "stride_key".to_string(),
                "somevalue".to_string()
            )]
        );
    }

    #[test]
    fn input_with_invalid_param() {
        let input = "# comment\n#x foobar\n";
        let mut visitor = TestVisitor::default();
        let mut reader = InstanceReader::new(&mut visitor);
        let res = reader.read(input.as_bytes());
        if let Err(ReaderError::InvalidParameterLine { lineno }) = res {
            assert_eq!(lineno, 1);
        } else {
            panic!("Wrong error");
        }
    }

    #[test]
    fn input_with_unknown_param() {
        let input = "# comment\n# another comment\n#x foobar []\n";
        let mut visitor = TestVisitor::default();
        let mut reader = InstanceReader::new(&mut visitor);
        let res = reader.read(input.as_bytes());
        if let Err(ReaderError::UnknownParameter { lineno, key }) = res {
            assert_eq!(lineno, 2);
            assert_eq!(key, String::from("foobar"));
        } else {
            panic!("Wrong error");
        }
    }

    #[test]
    fn input_with_parameter_with_invalid_json() {
        let input = "#x treedecomp [\n";
        let mut visitor = TestVisitor::default();
        let mut reader = InstanceReader::new(&mut visitor);
        let res = reader.read(input.as_bytes());
        assert!(matches!(res, Err(ReaderError::InvalidJSON { .. })));
    }

    #[test]
    fn input_with_tree_decomp() {
        let input = "#p 2 3\n#s stride_key somevalue\n(1);\n#x treedecomp [42,[[1,2],[3,4,5]],[[1,2],[3,4],[5,6]]]\n";

        let mut visitor = TestVisitor::default();
        let mut reader = InstanceReader::new(&mut visitor);
        reader.read(input.as_bytes()).unwrap();

        assert_eq!(
            visitor.param_tree_decomp,
            Some((
                3,
                TreeDecomposition {
                    treewidth: 42,
                    bags: vec![vec![1, 2], vec![3, 4, 5]],
                    edges: vec![(1, 2), (3, 4), (5, 6)]
                }
            ))
        );
    }
}
