#![cfg_attr(feature = "specialization",
    feature(specialization),
)]
#![allow(unused)]#![warn(unused_must_use)]

#[macro_use]
extern crate inheritance;

#[inheritable]
trait IsPoint {
    fn x (self: &'_ Self)
      -> f32
    ;

    fn y (self: &'_ Self)
      -> f32
    ;

    fn name (self: &'_ Self)
      -> String
    {
        format!("Point({x}, {y})", x = self.x(), y = self.y())
    }
}

struct Point {
    x: f32,
    y: f32,
}

impl IsPoint for Point {
    #[inline]
    fn x (self: &'_ Self) -> f32
    {
        self.x
    }

    #[inline]
    fn y (self: &'_ Self) -> f32
    {
        self.y
    }
}

#[derive(Inheritance)]
struct NewPoint (#[inherits(IsPoint)] Point);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Red,
    Green,
    Blue,
}

#[inheritable]
trait Colored {
    fn color (self: &'_ Self)
      -> Color
    ;
}

impl Colored for Color {
    #[inline]
    fn color (self: &'_ Self)
      -> Color
    {
        *self
    }
}

#[derive(Inheritance)]
struct NamedPoint {
    #[inherits(IsPoint)]
    point: Point,

    name: &'static str,

    #[inherits(Colored)]
    pigments: Color,
}

fn main ()
{
    let point = NamedPoint {
        point: Point { x: 42., y: 27. },
        name: "Carré",
        pigments: Color::Red,
    };
    dbg!(point.color());

    #[cfg(not(feature = "specialization"))] {
        dbg!(point.name());
    }
    #[cfg(feature = "specialization")] {
        // With the (nightly) specialization feature we can override the
        //
        impl IsPoint for NamedPoint {
            fn name (self: &'_ Self) -> String
            {
                format!("{} named {}", self.point.name(), self.name)
            }

            /// This, however, does not change the value returned by
            /// `self.point.name()` (obviously?)
            fn x (self: &'_ Self) -> f32
            {
                -1.
            }
        }
        dbg!(point.name());
        // Access the "parent" / non-specialized `impl` through an explicit delegation
        dbg!(point.point.name());
    }
}
