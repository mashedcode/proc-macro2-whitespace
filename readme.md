# proc-macro2-whitespace

## Usage

```
[dependescies]
proc-macro2-whitespace = { git = "https://github.com/mashedcode/proc-macro2-whitespace" }
```

```
use proc_macro2_whitespace::IntoCode;

let code = "pub fn foo() {\n    let foo = 'a';\n\n    let bar = 'b';\n}\n";
let stream = code.parse::<proc_macro2::TokenStream>().unwrap();
assert_eq!(stream.into_code().unwrap(), code);
```
