use crate::diagnostics::humane_code;

pub fn run_explain(code: &str) -> Result<(), String> {
    let normalized = normalize_code(code);
    let Some(explanation) = get_explanation(normalized) else {
        return Err(format!(
            "unknown diagnostic code `{code}`.\n\nTry a code like `S1002`, `S2002`, or `SNASK-SEM-TYPE-MISMATCH`."
        ));
    };
    println!("{}", explanation.trim());
    Ok(())
}

fn normalize_code(code: &str) -> &str {
    let trimmed = code.trim();
    match trimmed {
        "SNASK-PARSE-MISSING-RPAREN" => humane_code(trimmed),
        "SNASK-PARSE-MISSING-RBRACKET" => humane_code(trimmed),
        "SNASK-PARSE-MISSING-RBRACE" => humane_code(trimmed),
        "SNASK-PARSE-INDENT" => humane_code(trimmed),
        "SNASK-PARSE-SEMICOLON" => humane_code(trimmed),
        "SNASK-PARSE-EXPR" => humane_code(trimmed),
        "SNASK-PARSE-EXPECTED" => humane_code(trimmed),
        "SNASK-PARSE-TOKENIZE" => humane_code(trimmed),
        "SNASK-SEM-VAR-REDECL" => humane_code(trimmed),
        "SNASK-SEM-VAR-NOT-FOUND" => humane_code(trimmed),
        "SNASK-SEM-FUN-REDECL" => humane_code(trimmed),
        "SNASK-SEM-FUN-NOT-FOUND" => humane_code(trimmed),
        "SNASK-SEM-UNKNOWN-TYPE" => humane_code(trimmed),
        "SNASK-SEM-MISSING-RETURN" => humane_code(trimmed),
        "SNASK-SEM-TYPE-MISMATCH" => humane_code(trimmed),
        "SNASK-SEM-INVALID-OP" => humane_code(trimmed),
        "SNASK-SEM-IMMUTABLE-ASSIGN" => humane_code(trimmed),
        "SNASK-SEM-RETURN-OUTSIDE" => humane_code(trimmed),
        "SNASK-SEM-ARG-COUNT" => humane_code(trimmed),
        "SNASK-SEM-NOT-INDEXABLE" => humane_code(trimmed),
        "SNASK-SEM-INDEX-TYPE" => humane_code(trimmed),
        "SNASK-SEM-PROP-NOT-FOUND" => humane_code(trimmed),
        "SNASK-SEM-NOT-CALLABLE" => humane_code(trimmed),
        "SNASK-SEM-RESTRICTED-NATIVE" => humane_code(trimmed),
        "SNASK-BUILD-STANDARD-RUNTIME" => humane_code(trimmed),
        "SNASK-BUILD-BAREMETAL-BACKEND" => humane_code(trimmed),
        "SNASK-TINY-DISALLOWED-LIB" => humane_code(trimmed),
        _ => trimmed,
    }
}

pub fn get_explanation(code: &str) -> Option<&'static str> {
    match code {
        "S1002" => Some(
            "S1002: missing closing `)`

Snask found a function call or grouped expression that started with `(` but did not find the matching `)`.

Example:
  print(\"Hello\"

Fix:
  print(\"Hello\")

Tip:
  Look slightly before the highlighted location. The opening `(` is usually on the same line.",
        ),
        "S1003" => Some(
            "S1003: missing closing `]`

A list, index access, or type expression opened `[` but did not close it.

Example:
  let values = [1, 2, 3

Fix:
  let values = [1, 2, 3]",
        ),
        "S1004" => Some(
            "S1004: missing closing `}`

A brace block opened `{` but did not close it. Snask supports indentation-first code, so prefer indentation unless braces are intentional.",
        ),
        "S1005" => Some(
            "S1005: expected an indented block

Snask uses indentation to define blocks. After declarations like `class`, `fun`, `if`, `while`, `for`, and `zone`, the next line must be indented.

Example:
  fun start()
  print(\"Hello\")

Fix:
  fun start()
      print(\"Hello\")",
        ),
        "S1010" => Some(
            "S1010: expected an expression

Snask expected a value, variable, function call, object creation, or another expression here.

Example:
  let x =

Fix:
  let x = 10",
        ),
        "S1011" => Some(
            "S1011: unexpected token

The parser found a token that does not fit the grammar at this location. Check for a missing delimiter, an unfinished expression, or a statement in the wrong place.",
        ),
        "S2002" => Some(
            "S2002: variable not found

You are using a name that is not visible in the current scope. This is often a typo or a variable declared inside another block.

Example:
  let message = \"Hello\"
  print(mesage)

Fix:
  print(message)",
        ),
        "S2004" => Some(
            "S2004: function not found

Snask could not find a function with this name in the current scope. Check spelling, imports, and whether the function belongs to a namespace.",
        ),
        "S2010" => Some(
            "S2010: type mismatch

Snask found a value of one type where another type was expected.

Example:
  let age: int = \"18\"

Fix:
  let age: int = 18",
        ),
        "S2012" => Some(
            "S2012: immutable assignment

Variables declared with `let` cannot be reassigned. Use `mut` when the value is meant to change.

Example:
  let count = 0
  count = 1

Fix:
  mut count = 0
  count = 1",
        ),
        "S8001" => Some(
            "S8001: standard runtime required

You are compiling with `--profile baremetal`, but the highlighted code needs Snask's normal std/runtime layer.

In baremetal mode there is no default stdout, stdin, filesystem, OS process, GUI, libc, or C interop runtime.

Example:
  print(\"Hello\")

Fix:
  use a serial/VGA driver, or build with `--profile humane`

For low-level work that still wants the normal runtime, use:
  snask build app.snask --profile systems",
        ),
        "S8002" => Some(
            "S8002: baremetal backend not implemented yet

Snask recognizes the `baremetal` profile, but the freestanding backend still needs no_std/no_runtime support, custom entrypoints, linker scripts, and target-specific runtime rules.

Temporary fix:
  use `--profile systems` while baremetal support is being implemented.",
        ),
        "SNASK-PARSE-EXPR" => Some(
            "In Snask, when you declare a variable with 'let', you are creating a binding.

             This binding must be initialized immediately with a value because Snask variables are immutable by default.


             Example:

               let x = 10;      // Correct: 'x' is bound to the value 10.

               let x;           // Error: 'x' has no value.

               let x =          // Error: The expression ended abruptly.
"
        ),
        "SNASK-PARSE-SEMICOLON" => Some(
            "Statements in Snask (like variable declarations or function calls) must end with a semicolon ';'.\n\
             This tells the compiler where one instruction ends and the next begins.\n"
        ),
        "SNASK-SEM-TYPE-MISMATCH" => Some(
            "Snask is a statically typed language. This means every value has a specific type (like Number or String),\n\
             and you cannot mix them in ways that don't make sense (like adding a Number to a String).\n\n\
             Why? This prevents common bugs where you might accidentally try to perform math on text.\n\n\
             Example:\n\
               let x = 10 + \"hello\"; // Error: Type mismatch (Number vs String)\n\
               let x = 10 + 5;       // Correct: Both are Numbers.\n"
        ),
        "SNASK-SEM-VAR-REDECL" => Some(
            "In Snask, you cannot declare two variables with the same name in the same scope.\n\n\
             Why? If you have two variables named 'x', the compiler wouldn't know which one you are referring to later.\n\n\
             Example:\n\
               let x = 10;\n\
               let x = 20; // Error: 'x' is already declared.\n"
        ),
        "SNASK-SEM-VAR-NOT-FOUND" => Some(
            "You are trying to use a variable that hasn't been declared yet or is not visible in this scope.\n\n\
             In Snask, scopes are defined by indentation. A variable declared inside a block (like an 'if' or 'fun')\n\
             is not visible outside that block.\n\n\
             Check for typos or ensure the variable is declared before you use it.\n"
        ),
        "SNASK-SEM-IMMUTABLE-ASSIGN" => Some(
            "By default, variables declared with 'let' in Snask are immutable (they cannot be changed).\n\n\
             Why? Immutability makes your code safer and easier to reason about because you know a value won't change unexpectedly.\n\n\
             How to fix: If you need to change the value later, declare it with 'mut' instead of 'let'.\n\n\
             Example:\n\
               mut x = 10;\n\
               x = 20; // Correct: 'x' is mutable.\n"
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::get_explanation;

    #[test]
    fn explain_has_entry_for_missing_paren() {
        let text = get_explanation("S1002").expect("S1002 should be documented");
        assert!(text.contains("missing closing `)`"));
        assert!(text.contains("Fix:"));
    }

    #[test]
    fn explain_has_entry_for_unknown_variable() {
        let text = get_explanation("S2002").expect("S2002 should be documented");
        assert!(text.contains("variable not found"));
        assert!(text.contains("print(message)"));
    }
}
