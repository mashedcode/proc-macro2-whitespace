//! Serialize TokenStream into Rust Source Code while preserving the LineColumn information
//! provided by TokenStream Span.

use proc_macro2::{Delimiter, LineColumn, TokenStream, TokenTree};
use rustfmt_nightly as rustfmt;
use std::mem;
use std::result::Result;

/// # Example
///
/// ```
/// use proc_macro2_whitespace::IntoCode;
///
/// let code = "pub fn foo() {\n    let foo = 'a';\n\n    let bar = 'b';\n}\n";
/// let stream = code.parse::<proc_macro2::TokenStream>().unwrap();
/// assert_eq!(stream.into_code().unwrap(), code);
/// ```
pub trait IntoCode {
    fn into_code(self) -> Result<String, rustfmt::ErrorKind>;
}

trait IntoCodeHelper {
    fn into_code_with_original_whitespace(self, code: &mut String, cursor: &mut LineColumn);
}

fn rustfmt(code: String) -> Result<String, rustfmt::ErrorKind> {
    let mut config = rustfmt::Config::default();
    config.set().edition(rustfmt::Edition::Edition2018);
    config.set().emit_mode(rustfmt::EmitMode::Stdout);
    config.set().verbose(rustfmt::Verbosity::Quiet);
    let mut out = Vec::with_capacity(code.len());
    {
        let mut session = rustfmt::Session::new(config, Some(&mut out));
        session.format(rustfmt::Input::Text(code))?;
    }
    Ok(std::string::String::from_utf8(out).unwrap())
}

impl IntoCode for TokenStream {
    fn into_code(self) -> Result<String, rustfmt::ErrorKind> {
        let mut cursor = LineColumn { line: 1, column: 0 };
        let mut code = String::new();
        self.into_code_with_original_whitespace(&mut code, &mut cursor);
        rustfmt(code)
    }
}

impl IntoCodeHelper for TokenStream {
    fn into_code_with_original_whitespace(self, code: &mut String, cursor: &mut LineColumn) {
        for token in self.into_iter() {
            let span = token.span();
            match token {
                TokenTree::Group(group) => {
                    let span = group.span_open();
                    fill_whitespace(cursor, &span.start(), code);
                    mem::swap(cursor, &mut {
                        let mut end = span.start();
                        end.column += 1;
                        end
                    });
                    let delimiter = group.delimiter();
                    let delim_open = match delimiter {
                        Delimiter::Parenthesis => '(',
                        Delimiter::Brace => '{',
                        Delimiter::Bracket => '[',
                        Delimiter::None => 'Ø',
                    };
                    code.push(delim_open);
                    group
                        .stream()
                        .into_code_with_original_whitespace(code, cursor);
                    let span = group.span_close();
                    fill_whitespace(
                        cursor,
                        &{
                            let mut end = span.end();
                            if end.column > 0 {
                                end.column -= 1;
                            }
                            end
                        },
                        code,
                    );
                    mem::swap(cursor, &mut span.end());
                    let delim_close = match delimiter {
                        Delimiter::Parenthesis => ')',
                        Delimiter::Brace => '}',
                        Delimiter::Bracket => ']',
                        Delimiter::None => 'Ø',
                    };
                    code.push(delim_close);
                }
                token => {
                    let start = &span.start();
                    if let TokenTree::Ident(_) = token {
                        if cursor.line == 1
                            && cursor.column == 0
                            && start.line == 1
                            && start.column == 0
                        {
                            code.push(' ');
                        }
                    }
                    fill_whitespace(cursor, start, code);
                    code.push_str(&token.to_string());
                    mem::swap(cursor, &mut span.end());
                }
            }
        }
    }
}

fn fill_whitespace(prev: &LineColumn, curr: &LineColumn, code: &mut String) {
    if prev.line == 1 && prev.column == 0 && curr.line == 1 && curr.column == 0
        || prev.line > curr.line
        || (prev.line == curr.line && prev.column > curr.column)
    {
        return;
    }
    let mut whitespace = LineColumn {
        line: curr.line - prev.line,
        column: if curr.line > prev.line {
            curr.column
        } else {
            curr.column - prev.column
        },
    };
    while whitespace.line > 0 {
        code.push('\n');
        whitespace.line -= 1;
    }
    while whitespace.column > 0 {
        code.push(' ');
        whitespace.column -= 1;
    }
}
