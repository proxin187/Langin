# Langin

## Warning: Langin is still in early development and may have bugs

A c-like compiled programming language.

- [x] Type checking
- [ ] Standard library: there is currently no standard library
- [ ] Self hosted: I have plans of it compiling itself
- [ ] Assembly optimization

## Examples

Simple program that returns with 0 exit code:

```
main :: () -> int {
    return 0;
}
```

## Testing

Langin comes with tests provided to test if all the compiler features work correctly in the `./tests` folder. It is recommended to run the tests before building to compiler to be sure you have a working version of the compiler.

Run the tests with the command shown below:

```
$ python test.py
```

If all the tests ran successfully the output should look like this:

```
[TESTS]: successfully ran all tests in `./tests`
```

## Language specifications

In the language specifications you can find the specifications for syntax, types and behavior.

#### Binary expressions

A binary expression is a math expression consisting of left/right expressions and an operator such as `+`, `-`, `*`, `/`

Example:

```
34 + 35
```

In Langin the order of operations is very simple, the order of execution follows a recursive pattern where recursion is always on the right side.

Example:

```
34 + 35 * 2 + 8 == 34 + (35 * (2 + 8)) # NOT VALID SYNTAX #
```

#### Functions

A function is a callable piece of code which can accept up to 6 arguments and can return a value back to the caller.

Example:

```
square :: (num1 -> int, num2 -> int) -> int {
    return num1 * num2;
}
```

#### Variables

A variable is a value paired with a identifier used to reference it, variables are used to store values and have easy access to them, in Langin variables are stored localy on the stack.

Example:
```
let num -> int = 34 + 35;
```

#### If

An if statement is a type of conditional block consisting of a condition and a body, if the condition is true the body will be executed and if the condition is false the body will be skipped.

Example:
```
let example -> int = 69;
if example != 69 {
    return 420;
}
```

#### Else

An else statement provides extended control flow for if statements, in the previous paragraph we talked about if statements only executing the body if a condition is true and it skipping the body if the condition is false, with an else statement it will execute the else body if the condition is false.

Example:
```
let example -> int = 69;
if example != 69 {
    return 420;
} else {
    return 69;
}
```

#### Else If

In previous paragraphs i talked about if and else statements, these two are really the only thing you need and they are able to do any thing but if you have multiple conditions there will fast become a lot of nesting of if statements which will make your code unreadable and ugly, therefore else if statements prevent this nesting allowing your code to be both more readable and prettier.

Example:
```
let example -> int = 69;
if example != 69 {
    return 420;
} else if example == 48 {
    return 88;
} else {
    return 69;
}
```

#### While

While loops are a crucial part of programming because it allows the programmer to do things multiple times without repeating itself. while loops just like if statements consists of a condition and body, the while loop will continue to execute the body in a loop until the condition is false.

Example:
```
let example -> int = 0;
while example != 10 {
    example = example + 1;
}
```


#### Types

| Type    | Description                                                                                  |
| ---     | ---                                                                                          |
| `int`   | 64bit unsigned integer.                                                 |
| `ptr`  | pointer able point to any type.                                                     |
| `void`  | 0 bit type.                                                          |

#### Operators

Operators are used in the specified format: `lexpr [Op] rexpr`

| Op    | Description                                                                                  |
| ---     | ---                                                                                          |
| `+`  | get the sum of two values.                                                 |
| `-`  | subtract a value from another value.                                                     |
| `*`  | multiply a value with another value.                                                          |
| `/`  | Divide a value with another value.                                                          |





