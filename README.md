# LLamac
LLVM based toy language.(WIP)

***NOTE:** It is a work in progress and till now, only the lexer and some of the parser is done.
This is just a remote stash (currently).*

# Requirements
* LLVM 13.0 installed

# LLama Syntax
* Note: The syntax is subject to change.
* Functions:
    * Regular function
        ```
        def sum(a:i32, b:i32) -> i32 {
            return a + b;
        }
        ```
    * External functions
        ```
        extern sin(num:f32);
        ```
    * Calling a function
        ```
        sum(a, b);
        ```


* Variables:
    * Declaration with value
        ```
        let a:i32 = 5;
        ```
    * Declaration without value (will be assigned to trash value)
        ```
        let a:i32;
        ```
    * Referencing a variable
        ```
        let b:i32 = a + 5;
        ```
    * Reassign
        ```
        b = a -1;
        
        ```
* Operations
    * Available operations `=`, `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `+=`, `-=`, `*=`, `/=`
        ```
        let a:i32 = (-b + 5) - 10 / -(5 - -2);
        ```
* Comments
    * Comments start with `#` and go until the end of the line

* Programs
    * A program consists of just top-level functions, `extern` definitions, and expressions.
    * `main` function is the entry point

