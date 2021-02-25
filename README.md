# Yep: Your enjoyable programming language

This project is in very early stages. Expect frequent breaking changes.

## REPL

In the REPL (**r**ead-**e**val-**p**rint-**l**oop) you can type Yep code, which is ~~evaluated~~ parsed as soon as you press <kbd>Enter</kbd> twice. Note that Yep can't execute any code yet. I'm currently working on the parser and type checker.

To install the REPL, [git](https://git-scm.com/) and  [cargo](https://github.com/rust-lang/cargo/) must be installed. Then run

```fish
git clone https://github.com/Aloso/yep
cargo install --path yep/crates/repl
```

You might have to add `~/.cargo/bin` to your PATH variable.

## Introduction

Yep is a statically-typed multi-paradigm programming language. It is inspired by and implemented in Rust.

Yep tries to bring Rust's ergonomics to a simpler, higher-level language. It trades a bit of performance for a lower learning-curve. For instance, it doesn't have concepts like lifetimes or borrowing. Types are automatically boxed or reference-counted when necessary, but most types can be passed by value.

## Syntax

The syntax is inspired by Rust and some functional programming languages. It is whitespace-insensitive. Comments start with `#`.

### Expressions

Yep is expression-based. Everything within a function is an expression. For example, a block is an expression that evaluates to its last expression:

```ruby
let x = {
    let a = +6;
    let b = +7;
    a * b
};
```

Parentheses (`()`) are used solely for function calls and tuples. To group expressions together, use braces:

```ruby
let distance = {{x * x} + {y * y}}.sqrt;
```

In yep, operators such as `*` and `+` are just regular functions. Every function that starts with an operator (`+-*/%~<>=`) can be used as an infix operator:

```ruby
let power = a ** 4;
let map = hashmap("hello" -> "world", "foo" -> "bar");
let cross-product = (1, 2, 3) *cross (4, 5, 6);
```

Note that Yep is unaware of arithmetic precedence rules, so you have to use braces when an expression is ambiguous:

```ruby
let y = a * x + b;   # error: Evaluation order must be disambiguated with a block

let y = {a * x} + b  # ok
```

Also note that `+-*/%~<>=` symbols can appear in a name, so `x-y` is not the difference of `x` and `y`, it is just one name.

### Functions

A function is defined with the `fun` keyword:

```clojure
fun do-something(x Int) Int {
    x * 42
}
```

This defines a function called `do-something`. It has one parameter, `x`, of the type `Int`. The return type is `Int`.

If a function should return nothing, use the `Unit` return type:

```ruby
fun print(x Int, y Int) Unit {
    print(x);
    print("/");
    print(y);
}
```

There are three ways to invoke a function:

#### Regular functions

Regular functions are invoked by writing the function arguments in parentheses after the function name:

```ruby
print(4, 2)
```

Naming the argument names is allowed too:

```ruby
print(x: 4, y: 2)
```

If the function has no arguments, the parentheses are optional.

#### Functions with receiver

A function has a "receiver" if its first argument is called `self`:

```ruby
fun foo(self Int, other Bool) Unit {}
```

Functions with receiver are used like this:

```ruby
42.foo(true)
```

I.e., the first argument is written before the dot, the remaining arguments are provided in the parentheses.

If the function has only one argument, the parentheses can be omitted, e.g. `42.foo`.

#### Operator functions

An operator (or infix) function is one that starts with an operator and has exactly two arguments. These are usually invoked by writing the name between the arguments:

```ruby
5 + 3;
(1, 2, 3) *cross (4, 5, 6);
"hello" -> "world";
```

However, they can also be used like regular functions:

```ruby
+(5, 3);
*cross((1, 2, 3), (4, 5, 6));
->("hello", "world");
```

### Tuples

A tuple is an ordered collection of values. They can have different types:

```ruby
let one_tuple = (5);
let six_tuple = (1, 1, +2, true, "hello", (3));
```

If a tuple contains only values of the same type, it is an array:

```ruby
let array = (1, 1, 2, 3, 5, 8, 13);
```

Arrays can be indexed by calling them as a function:

```ruby
let array = (1, 1, 2, 3, 5, 8, 13);
let n = 3;
let fourth = array(n); # indexing is zero-based
```

This only works for arrays, not for other tuples, because otherwise then the return type would depend on the index, which might not be known at compile time. Therefore, the only way to access fields of a tuple is to _destructure_ it:

```ruby
let six_tuple = (1, 1, +2, true, "hello", (3));
let (a, _, _, d, _, f, (g)) = six_tuple;
# a is set to 1
# d is set to true
# f is set to "hello"
# g is set to 3
```

The type of a tuple is written as `Tuple[First, Second, Third]`. If the tuple is an array, it can be written as `Tuple[n * T]`, where `n` is a number and `T` is the inner type.

Note that both tuples and arrays have a fixed size. For lists, it's better to use a library type like `Vec[T]`.

### Types

Types are always uppercase. This is enforced by the compiler to make it easier to reason about code.

Yep has several built-in types, which are:

| Type    | Description                  | Literal |
| ------- | ---------------------------- | --- |
| `Int`   | Signed 64-bit integer        | `5`, `-5` |
| `I32`   | Signed 32-bit integer
| `I16`   | Signed 16-bit integer
| `I8`    | Signed 16-bit integer
| `UInt`  | Unsigned 64-bit integer      | `+5` |
| `U32`   | Unsigned 32-bit integer
| `U16`   | Unsigned 16-bit integer
| `U8`    | Unsigned 16-bit integer
| `Float` | 64-bit floating-point number | `5.0` |
| `F32`   | 32-bit floating-point number
| `Char`  | UTF-8 code point             | `'h'` |
| `String` | UTF-8 encoded text          | `"hello"` |

Number literals are 64-bit by default. To create numbers with less precision, you can write the type after the literal, e.g.

```ruby
let x = 5 I32;
```

#### Classes

A class is a type that can contain data and have associated behaviour. This behaviour is defined with functions, called methods:

```ruby
class Point(x Int, y Int);

impl Point {
    fun +(self Point, other Point) Point {
        Point(self.x + other.x, self.y + other.y)
    }
}
```

The above class `Point` has two fields, `x` and `y`. A field is very similar to a function with receiver: It can be accessed with `object.field_name`. A class object is created by calling its constructor (a special function with the same name as the class) with the fields as arguments, e.g. `Point(x: 5, y: 7)`.

The `impl` block defines associated behaviour for the class. The above `impl` block has a method called `+` for adding two points:

```ruby
let p1 = Point(x: 4, y: 2);
let p2 = Point(x: 6, y: 3);
let p3 = p1 + p2;
# or:
let p3 = p1.+(p2);
```

#### Enums

```ruby
enum Result[T, E] {
    ok(value T),
    err(error E),
}
```

Enums (called "sum types" in type theory) are types with multiple constructors, called _variants_. In the example above, the variants are `ok` and `err`, so the type can contain _either_ a value _or_ an error.

An enum object is created by calling one of its variants:

```ruby
let ok_object = Result.ok(5);
let err_object = Result.err("something unexpected happened");
```

To check of which variant an enum object is, a `match` block is needed:

```ruby
let object Result[Int, String] = ...;
object match {
    Result.ok(value): print(value),
    Result.err(error): print_error(error),
};
```

On the left of the colons (`:`) are _patterns_. In the `match` block, the object is matched against consecutively against each pattern. As soon as a pattern matches, the expression on the right is executed. There is no implicit fall-through.

Furthermore, the `match` block must be _exhaustive_: Every possible value must be covered by the match patterns. This has the benefit that `match` arms can return a value:

```ruby
let x = object match {
    Result.ok(0 | 1 | 2): 2,
    Result.ok(x): x + 1,
    Result.err(error): return,
};
```

Here, the first pattern matches against specific numbers (0, 1 or 2). The second pattern matches against any number `x` that is in a `Result.ok` enum. Omitting the 2nd or 3rd pattern would cause a compilation error, since not all possible values would be covered.

Not every enum variant has to contain a value. For example, the type `Bool` is defined like this:

```ruby
enum Bool {
    true,
    false,
}
```

#### Traits

A trait is like an interface in object-oriented languages. It defines shared behaviour and can be implemented by other types. A type can implement any number of traits. Example:

```ruby
trait Default {
    fun default() Self;
}

class Foo();

impl Default for Foo {
    fun default() Self {
        Foo()
    }
}
```

The trait defines a function, `default`, that doesn't have a body. It's only a _signature_ (the function name, arguments and return type). The class `Foo` implements this trait with an `impl` block that contains the `default` function.

The `Self` type is special: It refers to the type that implements the trait. So, when implementing `Default` for `Foo`, then `Self` is an alias for `Foo`.

#### Generics

With generics, it's possible to generalize over types that have similar characteristics.

For example, let's write a function that accepts any numeric type (`Int`, `I32`, `I16`, `I8`, `UInt`, `U32`, `U16`, `U8`, `Float`, `F32`). Fortunately, all these types implement the `Num` trait:

```ruby
fun double[N impl Num](self N) N {
    self * N.two
}
```

#### Closures

A closure is an anonymous function that can be passed around and invoked later. It allows to "close over" (i.e., access) local variables. Closures are list of arguments enclosed between vertical pipes (`|`) followed by the closure body, which is either a `{...}` block or a bare expression:

```ruby
fun run_twice[T](arg T, closure |T| Unit) Unit {
    closure(arg);
    closure(arg);
}

run_twice(arg: 7, closure: |a| print(a + 42));

var x = 5;
run_twice(arg: 17, closure: |s| {
    x *= 2;
})
```

By default, closures can _escape_. This is best explained with an example:

```ruby
fun do_something(closure || Unit) Unit {
    print("hello");
    closure();
    print("world");
}
```

You might expect that this function will always print "hello" and "world" (unless the closure raises a panic). However, the closure is also able to escape with simple control flow, like a `break` or `return` leading out of the closure:

```ruby
fun main() I32 {
    do_something(|| {
        return 42
    });
    0
}
```

This `return` statement exits the `main` function, not just the closure. This is useful in many situations, but it's not always desired: In the above example, "hello" is printed but "world" isn't, even though no error is raised. There are two solutions:

### The `defer` keyword

The `defer` keyword defines an expression that is executed when the current scope is exited. It is executed no matter in which way it is exited: Normally, via `return`, `break`, `continue`, escape, error, or panic:

```ruby
fun do_something(closure || Unit) Unit {
    print("hello");
    defer print("world");
    closure();
}
```

### The `noescape` keyword

Adding the `noescape` keyword to a closure's type makes it impossible to escape from it:

```ruby
fun do_something(closure noescape || Unit) Unit {
    print("hello");
    closure();
    print("world");
}
```

Now trying to call this function with a closure that can escape will produce a compiler error.

## Control flow expressions

The following keywords:

* `return`
* `break`
* `continue`

work the same as in most other languages. For branching, yep uses the `match` keyword (see above), and will eventually support `if`/`else` and `while`. It will also have a `loop` and a `for` expression like Rust.

For logic expressions, it has `and` and `or` keywords like in python. It doesn't have a `not` keyword, it uses a function for this:

```ruby
let is_it_true = foo and bar() or baz.not;
```

`and` and `or` are keywords, because they are short-circuiting, so they can't be implemented as a function, but `not` can.

An `is` keyword for ad-hoc pattern matching that supports variable bindings is planned:

```ruby
let x = some(5);
let y = if (x is some(inner) and inner > 5) inner else 0;
```

## Attributes

Attributes allow to attach further information to an item. This information can be read by the compiler. They will also be used by macros, if they ever get implemented. Attributes start with a `@`, followed by an identifier and optional parentheses that can contain values:

```rust
@foo(bar)
fun hello_world() @foo Int {
    3 @baz
}

class MyClass(
    @foo x Int = 3 @baz,
)
```

Note that the attributes go _before_ an item/type, but _after_ an expression. The reason that attributes appear before items is that people are used to that from many other languages. For expressions, however, postfix attributes are more ergonomic and avoid some syntactic ambiguity.

The parentheses can contain a list of comma-separated expressions. For example:

```ruby
@deprecated(reason = "Not very precise. Use `other-pi` instead")
let pi = 3.14;

@allow(unused-variables)
fun f(x Int) Unit {}
```

# TODO

- `defer` statement
- parse closure types, with `noescape` keyword
- Parse attributes
