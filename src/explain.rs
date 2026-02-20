pub fn get_explanation(code: &str) -> Option<&'static str> {
    match code {
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
