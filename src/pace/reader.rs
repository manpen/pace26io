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
///  fn visit_tree(&mut self, lineno: usize, line: &str) -> Action {
///    println!("Tree at line {}: {}", lineno + 1, line);
///    Action::Continue
///  }
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
    fn visit_header(&mut self, _lineno: usize, _num_trees: usize, _num_leafs: usize) -> Action {
        Action::Continue
    }
    fn visit_tree(&mut self, _lineno: usize, _line: &str) -> Action {
        Action::Continue
    }
    fn visit_line_with_extra_whitespace(&mut self, _lineno: usize, _line: &str) -> Action {
        Action::Continue
    }
    fn visit_unrecognized_dash_line(&mut self, _lineno: usize, _line: &str) -> Action {
        Action::Continue
    }
    fn visit_unrecognized_line(&mut self, _lineno: usize, _line: &str) -> Action {
        Action::Continue
    }
    fn visit_stride_line(&mut self, _lineno: usize, _line: &str, _key : &str, _value : &str) -> Action {
        Action::Continue
    }
}

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error("Identified line {} as header. Expected '#p {{numtree}} {{numleaves}}'", lineno+1)]
    InvalidHeaderLine { lineno: usize },

    #[error("Identified line {} as stride line. Expected '#s {{key}}: {{value}}'", lineno+1)]
    InvalidStrideLine { lineno: usize },

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

fn try_parse_stride_line(line: &str) -> Option<(&str, &str)> {
    let split = line.find(':')?;

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
        let mut header_line = None;
        for (lineno, line) in reader.lines().enumerate() {
            let line = line?;
            let content = line.trim();

            if content.len() != line.len() {
                // line has extra whitespace
                if self.visitor.visit_line_with_extra_whitespace(lineno, &line) == Action::Terminate
                {
                    return Ok(());
                }
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
                        if self.visitor.visit_header(lineno, num_trees, num_leaves)
                            == Action::Terminate
                        {
                            return Ok(());
                        }
                    } else {
                        return Err(ReaderError::InvalidHeaderLine { lineno });
                    }
                } else if content.starts_with("#s") {
                    // stride line in the format "#s key: value"
                    if let Some((key, value)) = try_parse_stride_line(content) {
                        if self.visitor.visit_stride_line(lineno, content, key, value) == Action::Terminate {
                            return Ok(());
                        }
                    } else {
                        return Err(ReaderError::InvalidStrideLine { lineno });
                    }
                } else {
                    // unrecognized line
                    if self.visitor.visit_unrecognized_dash_line(lineno, content)
                        == Action::Terminate
                    {
                        return Ok(());
                    }
                }
                continue;
            }

            if content.ends_with(";") {
                if self.visitor.visit_tree(lineno, content) == Action::Terminate {
                    return Ok(());
                }
                continue;
            }

            if self.visitor.visit_unrecognized_line(lineno, content) == Action::Terminate {
                return Ok(());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestVisitor {
        pub headers: Vec<(usize, usize, usize)>,
        pub trees: Vec<(usize, String)>,
        pub extra_whitespace_lines: Vec<(usize, String)>,
        pub unrecognized_dash_lines: Vec<(usize, String)>,
        pub unrecognized_lines: Vec<(usize, String)>,
        pub stride_lines: Vec<(usize, String, String, String)>,
    }

    impl TestVisitor {
        fn new() -> Self {
            Self {
                headers: vec![],
                trees: vec![],
                extra_whitespace_lines: vec![],
                unrecognized_dash_lines: vec![],
                unrecognized_lines: vec![],
                stride_lines: vec![],
            }
        }
    }

    impl InstanceVisitor for TestVisitor {
        fn visit_header(&mut self, lineno: usize, num_trees: usize, num_leafs: usize) -> Action {
            self.headers.push((lineno, num_trees, num_leafs));
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

        fn visit_unrecognized_dash_line(&mut self, lineno: usize, line: &str) -> Action {
            self.unrecognized_dash_lines
                .push((lineno, line.to_string()));
            Action::Continue
        }

        fn visit_unrecognized_line(&mut self, lineno: usize, line: &str) -> Action {
            self.unrecognized_lines.push((lineno, line.to_string()));
            Action::Continue
        }

        fn visit_stride_line(&mut self, lineno: usize, line: &str, key: &str, value: &str) -> Action {
            self.stride_lines.push((lineno, line.to_string(), key.to_string(), value.to_string()));
            Action::Continue
        }
    }

    #[test]
    fn test_valid_input() {
        let input = "#p 2 3\n(1);\n# comment\n(2);\n";

        let mut visitor = TestVisitor::new();
        let mut reader = InstanceReader::new(&mut visitor);
        reader.read(input.as_bytes()).unwrap();

        assert_eq!(visitor.headers, vec![(0, 2, 3)]);
        assert_eq!(
            visitor.trees,
            vec![(1, "(1);".to_string()), (3, "(2);".to_string())]
        );
        assert!(visitor.extra_whitespace_lines.is_empty());
        assert!(visitor.unrecognized_dash_lines.is_empty());
        assert!(visitor.unrecognized_lines.is_empty());
    }

    #[test]
    fn input_with_whitespace() {
        let input = "#p 2 3\n (1);\n\n(2);";

        let mut visitor = TestVisitor::new();
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
        assert!(visitor.unrecognized_dash_lines.is_empty());
        assert!(visitor.unrecognized_lines.is_empty());
    }

    #[test]
    fn input_with_unrecognized_lines() {
        let input = "#p 2 3\n (1);\n\n(2);\n#<illegal comment\n(3)missing semicolon";

        let mut visitor = TestVisitor::new();
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
            visitor.unrecognized_dash_lines,
            vec![(4, "#<illegal comment".to_string())]
        );
        assert_eq!(
            visitor.unrecognized_lines,
            vec![(5, "(3)missing semicolon".to_string())]
        );
    }

    #[test]
    fn input_with_stride_line() {
        let input = "#p 2 3\n#s stride_key: somevalue\n(1);\n";
        let mut visitor = TestVisitor::new();
        let mut reader = InstanceReader::new(&mut visitor);
        reader.read(input.as_bytes()).unwrap();

        assert_eq!(visitor.stride_lines,
                   vec![(1, "#s stride_key: somevalue".to_string(), "stride_key".to_string(), "somevalue".to_string())]);
    }
}
