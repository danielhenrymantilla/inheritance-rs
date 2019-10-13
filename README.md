# `::inheritance`

This (experimental) crate provides procedural macros to get "inheritance-like"
behavior by **deriving delegation from composition**.

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](https://github.com/danielhenrymantilla/inheritance-rs)
[![Latest version](https://img.shields.io/crates/v/inheritance.svg)](https://crates.io/crates/inheritance)
[![Documentation](https://docs.rs/inheritance/badge.svg)](https://docs.rs/inheritance)
[![License](https://img.shields.io/crates/l/inheritance.svg)](https://github.com/danielhenrymantilla/inheritance-rs/blob/master/LICENSE)


## Presentation

Imagine having some `Point` type and some behavior / associated methods:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: f32,

    y: f32,
}

impl Point {
    fn x (self: &'_ Self) -> f32
    {
        self.x
    }

    fn y (self: &'_ Self) -> f32
    {
        self.y
    }

    fn name (self: &'_ Self) -> Option<&'_ str>
    {
        Some("Point")
    }
}
```

Now imagine you want to have a new "`Point`-like" type, but with extra
attributes (and maybe some overriden behavior):

```rust
# struct Point;
struct NamedPoint {
    name: String,

    point: Point,
}
```

First of all, being "`Point`-like" requires abstracting the inherent methods
under a trait:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: f32,

    y: f32,
}

trait IsPoint {
    fn x (self: &'_ Self) -> f32;

    fn y (self: &'_ Self) -> f32;

    fn name (self: &'_ Self) -> Option<&'_ str>;
}

impl IsPoint for Point {
    fn x (self: &'_ Self) -> f32
    {
        self.x
    }

    fn y (self: &'_ Self) -> f32
    {
        self.y
    }

    fn name (self: &'_ Self) -> Option<&'_ str>
    {
        Some("Point")
    }
}
```

Now we'd like `NamedPoint` to implement `IsPoint`:

```rust
# #[derive(Debug,Clone,Copy,PartialEq)]struct Point{x:f32, y:f32}trait IsPoint{fn x(&self)->f32;fn y(&self)->f32;fn name(&self)->Option<&str>;}impl IsPoint for Point{fn x(&self)->f32{self.x}fn y(&self)->f32{self.y}fn name(&self)->Option<&str>{Some("Point")}}
struct NamedPoint {
    name: String,

    point: Point,
}

impl IsPoint for NamedPoint {
    #[inline]
    fn x (self: &'_ Self) -> f32
    {
        self.point.x()
    }

    #[inline]
    fn y (self: &'_ Self) -> f32
    {
        self.point.y()
    }

    #[inline]
    fn name (self: &'_ Self) -> Option<&'_ str>
    {
        self.point.name()
    }
}
```

This leads to writing very repetitive and uninteresting (delegation) code...

## Enter `::inheritance`

This last implementation can be completely skipped by slapping a
`#[inheritable]` attribute on the `IsPoint` trait, and a `Inheritance` derive
on the `NamedPoint` struct:

```rust
use ::inheritance::{inheritable, Inheritance};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: f32,

    y: f32,
}

#[inheritable]
trait IsPoint {
    fn x (self: &'_ Self) -> f32;

    fn y (self: &'_ Self) -> f32;

    fn name (self: &'_ Self) -> Option<&'_ str>;
}

impl IsPoint for Point {
    fn x (self: &'_ Self) -> f32
    {
        self.x
    }

    fn y (self: &'_ Self) -> f32
    {
        self.y
    }

    fn name (self: &'_ Self) -> Option<&'_ str>
    {
        Some("Point")
    }
}

#[derive(Inheritance)]
struct NamedPoint {
    name: String,

    #[inherits(IsPoint)]
    point: Point,
}
```

### `nightly` Rust

This feature is especially interesting with the `specialization` feature, which
you can already use in nightly.

When depending on the `inheritance` crate in your `Cargo.toml`, you can specify
that you want to use this feature:

```toml
[dependencies]
inheritance = { version = "...", features = ["specialization"] }
```

You will then be able to override some of the auto-generated delegation methods:

```rust
# use::inheritance::{inheritable, Inheritance};#[derive(Debug,Clone,Copy,PartialEq)]struct Point{x:f32, y:f32}#[inheritable]trait IsPoint{fn x(&self)->f32;fn y(&self)->f32;fn name(&self)->Option<&str>;}impl IsPoint for Point{fn x(&self)->f32{self.x}fn y(&self)->f32{self.y}fn name(&self)->Option<&str>{Some("Point")}}
#[derive(Inheritance)]
struct NamedPoint {
    name: String,

    #[inherits(IsPoint)]
    point: Point,
}

# #[cfg(feature = "specialization")]
impl IsPoint for NamedPoint {
    fn name (self: &'_ Self) -> Option<&'_ str>
    {
        Some(&*self.name)
    }
}
```

## Going further

#### Newtypes

The `#[inherits(...)]` field attribute can be used on tuple structs fields,
which makes it the perfect tool for newtypes:

```rust
# use::inheritance::{inheritable, Inheritance};#[derive(Debug,Clone,Copy,PartialEq)]struct Point{x:f32, y:f32}#[inheritable]trait IsPoint{fn x(&self)->f32;fn y(&self)->f32;fn name(&self)->Option<&str>;}impl IsPoint for Point{fn x(&self)->f32{self.x}fn y(&self)->f32{self.y}fn name(&self)->Option<&str>{Some("Point")}}
#[derive(Inheritance)]
struct AnonymousPoint (
    #[inherits(IsPoint)]
    Point,
);

# #[cfg(feature = "specialization")]
impl IsPoint for AnonymousPoint {
    fn name (self: &'_ Self) -> Option<&'_ str>
    {
        None
    }
}
```

#### Multiple "inheritance"

A single struct with a `#[derive(Inheritance)]` on it can have multiple
`#[inherits(...)]` on either the same field or multiple distinct fields, since
the implementation of each "inherited" trait will just be delegating the
methods of that trait onto the decorated field. If you try to "inherit" the same
trait from multiple fields, then classic coherence rules will prevent it:

```rust
# use::inheritance::{inheritable, Inheritance};#[derive(Debug,Clone,Copy,PartialEq)]struct Point{x:f32, y:f32}#[inheritable]trait IsPoint{fn x(&self)->f32;fn y(&self)->f32;fn name(&self)->Option<&str>;}impl IsPoint for Point{fn x(&self)->f32{self.x}fn y(&self)->f32{self.y}fn name(&self)->Option<&str>{Some("Point")}}
#[derive(Debug, Clone, Copy, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

#[inheritable]
trait Colored {
    fn color (self: &'_ Self) -> Color;
}

impl Colored for Color {
    #[inline]
    fn color (self: &'_ Self) -> Color
    {
        *self
    }
}

#[derive(Inheritance)]
struct ColoredPoint {
    #[inherits(IsPoint)]
    point: Point,

    #[inherits(Colored)]
    pigments: Color,
}
```

#### Virtual dispatch

The idea behind "virtual" methods, _à la_ C++, is that objects _always_ always
carry a vtable with _some_ methods in it, the ones marked `virtual` (and with
the other methods _never_ in it). In Rust, however, "objects" _sometimes_ carry
a vtable (_i.e._, `struct`s and `enum`s do not, but trait objects (`dyn Trait`)
do), and this vtable contains _all_ the methods of the trait and its
supertraits:

Imagine having a `.present()` method whose body calls `.name()`, and where the
behavior / value returned by the `.name()` call changes for each different
`Point`.


```rust,ignore
/// somewhere in the code
fn present (self: &'_ Self) -> Cow<'static, str>
// where Self : IsPoint
{
    if let Some(name) = self.name() {
        Cow::from(format!("{}({}, {})", name, self.x(), self.y()))
    } else {
        Cow::from("<anonymous>")
    }
}

let point = Point { x: 42., y: 27. };
assert_eq!(
    &point.present() as &str,
    "Point(42.0, 27.0)"
);

let anonymous_point = AnonymousPoint(point);
assert_eq!(
    &anonymous_point.present() as &str,
    "<anonymous>"
);

let named_point = NamedPoint { name: "Carré", point };
assert_eq!(
    &named_point.present() as &str,
    "Carré(42.0, 27.0)"
);
```

  - In a language such as C++, this can be achieved by tagging the `.name()`
    method as `virtual`, and by then having `.present()` be a method of the
    base class (not necessarily `virtual` itself).

  - In Rust the above strategy cannot be done; one must use a function / method
    polymorphic over implementors of the `IsPoint` type, using:

      - dynamic dispatch (the mechanism in C++ behind `virtual` methods):

        ```rust
        # use::inheritance::{inheritable, Inheritance};#[derive(Debug,Clone,Copy,PartialEq)]struct Point{x:f32, y:f32}#[inheritable]trait IsPoint{fn x(&self)->f32;fn y(&self)->f32;fn name(&self)->Option<&str>;}impl IsPoint for Point{fn x(&self)->f32{self.x}fn y(&self)->f32{self.y}fn name(&self)->Option<&str>{Some("Point")}}use::std::borrow::Cow;
        impl dyn IsPoint + '_ {
            fn present (self: &'_ Self) -> Cow<'static, str>
            {
                if let Some(name) = self.name() {
                    Cow::from(format!("{}({}, {})", name, self.x(), self.y()))
                } else {
                    Cow::from("<anonymous>")
                }
            }
        }
        ```

      - static dispatch; then getting method-like syntax requires a helper
        trait:

        ```rust
        # use::inheritance::{inheritable, Inheritance};#[derive(Debug,Clone,Copy,PartialEq)]struct Point{x:f32, y:f32}#[inheritable]trait IsPoint{fn x(&self)->f32;fn y(&self)->f32;fn name(&self)->Option<&str>;}impl IsPoint for Point{fn x(&self)->f32{self.x}fn y(&self)->f32{self.y}fn name(&self)->Option<&str>{Some("Point")}}use::std::borrow::Cow;
        trait Present
        where
            Self : IsPoint,
        {
            fn present (self: &'_ Self) -> Cow<'static, str>
            {
                if let Some(name) = self.name() {
                    Cow::from(format!("{}({}, {})", name, self.x(), self.y()))
                } else {
                    Cow::from("<anonymous>")
                }
            }
        }
        impl<T : ?Sized> Present for T
        where
            T : IsPoint,
        {}
        ```

Currently the macro does not target full-featured "virtual" methods, so
_overriding_ `.present()` itself is not yet possible.

Since this is an experimental crate, I will try to explore that possibility...
