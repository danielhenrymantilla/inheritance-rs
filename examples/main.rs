#![feature(specialization)]

#[macro_use]
extern crate inheritance;

use ::std::borrow::Cow;

#[inheritable]
trait Point {
    fn x (self: &'_ Self)
      -> f32
    ;
    fn y (self: &'_ Self)
      -> f32
    ;

    fn clear<T> (self: &'_ mut Self, _: ())
    ;

    fn name (self: &'_ Self) -> Cow<'_, str>
    {
        format!("Point({x}, {y})", x = self.x(), y = self.y()).into()
    }
}

struct Coords {
    x: f32,
    y: f32,
}

impl Point for Coords {
    #[inline] fn x (self: &'_ Self) -> f32 { self.x }
    #[inline] fn y (self: &'_ Self) -> f32 { self.y }
    #[inline] fn clear<T> (self: &'_ mut Self, _: ()) {}
}

#[derive(Inheritance)]
struct NewPoint (#[inherits(Point)] Coords);

#[derive(Inheritance)]
struct NamedPoint {
    #[inherits(self::Point)]
    coords: Coords,

    name: &'static str,
}

impl Point for NamedPoint {
    fn x (&self) -> f32 { -1. }
    fn name (self: &'_ Self) -> Cow<'_, str>
    {
        format!("{} named {}",
            self.parent().name(),
            self.name,
        ).into()
    }
}

fn main ()
{
    let point = NamedPoint {
        coords: Coords { x: 42., y: 27. },
        name: "Carré",
    };
    dbg!(point.name());
    dbg!(point.parent().name());
}
