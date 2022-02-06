mod brush;
mod entity;
mod plane;

use std::fs;
use std::path::Path;
use std::str::FromStr;

use nom::error::VerboseError;
use nom::error::convert_error;

use crate::Result;
use crate::error::Error;

use entity::Entity;

#[derive(Debug, Default)]
pub struct Map(pub Vec<Entity>);

impl Map {
    pub fn from_file<P>(path: P) -> Result<Self> where P: AsRef<Path> {
        Map::from_str(&fs::read_to_string(path)?)
    }
}

impl FromStr for Map {
    type Err = Error;

    fn from_str<'a>(s: &str) -> Result<Self> {
        match parser::parse_map::<VerboseError<&str>>(s) {
            Ok((_remainder, map)) => Ok(map),
            Err(nom::Err::Error(e))
            | Err(nom::Err::Failure(e)) => {
              Err(Error::ParseError(convert_error(s, e)))
            },
            Err(nom::Err::Incomplete(needed)) => {
                Err(Error::IncompleteData(format!("{needed:?}")))
            }
          }
    }
}

mod parser {
    use nom::IResult;
    use nom::bytes::complete::take_until;
    use nom::character::complete::char;
    use nom::character::complete::multispace0;
    use nom::combinator::map;
    use nom::error::ContextError;
    use nom::error::ParseError;
    use nom::error::context;
    use nom::multi::many0;
    use nom::number::complete::float;
    use nom::sequence::delimited;
    use nom::sequence::pair;
    use nom::sequence::preceded;
    use nom::sequence::tuple;

    use super::Map;
    use super::brush::Brush;
    use super::entity::Entity;
    use super::plane::Plane;

    fn ws<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
        multispace0(i)
    }

    fn string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
        delimited(char('"'), take_until("\""), char('"'))(i)
    }

    fn property<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, (String, String), E> {
        map(tuple((string, ws, string, ws)), |(key, _, value, _)| (key.to_string(), value.to_string()))(i)
    }

    fn num<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, f32, E> {
        float(i)
    }

    fn point<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, [f32; 3], E> {
        context(
            "point",
            delimited(
                pair(char('('), ws),
                map(tuple((num, ws, num, ws, num)), |(x, _, y, _, z)| [x, y, z]),
                pair(ws, char(')')),
            ),
        )(i)
    }

    fn tex<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
        take_until(" ")(i)
    }

    fn plane<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, Plane, E> {
        context(
            "plane",
            map(
                tuple((point, ws, point, ws, point, ws, tex, ws, num, ws, num, ws, num, ws, num, ws, num, ws)),
                |(a, _, b, _, c, _, texture, _, x_offset, _, y_offset, _, rotation, _, x_scale, _, y_scale, _)| {
                    Plane { points: [a, b, c], texture: texture.to_string(), x_offset, y_offset, rotation, x_scale, y_scale }
                },
            ),
        )(i)
    }

    fn brush<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, Brush, E> {
        context(
            "brush",
            delimited(
                pair(char('{'), ws),
                map(many0(plane), |brush| brush),
                pair(ws, char('}')),
            ),
        )(i)
    }

    fn entity<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, Entity, E> {
        context(
            "entity",
            delimited(
                pair(char('{'), ws),
                map(
                    tuple((
                        map(many0(property), |properties| properties.iter().cloned().collect()),
                        many0(brush),
                    )),
                    |(properties, brushes)| Entity { properties, brushes }
                ),
                tuple((ws, char('}'), ws)),
            ),
        )(i)
    }

    pub(super) fn parse_map<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, Map, E> {
        context(
            "parse_map",
            preceded(ws, map(many0(entity), |entities| Map(entities)))
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::str::FromStr;

    use crate::Map;

    const MAP_EMPTY_ENTITY: &str = "{}";
    const MAP_PROPERTY: &str = "{\"hello\" \"world\"}";
    const MAP_BRUSH: &str = "{{( -64 -64 -16 ) ( -64 -63 -16 ) ( -64 -64 -15 ) __TB_empty 0 0 0 1 1 }}";
    const MAP_SQUARE: &str = r#"
        {
        "classname" "worldspawn"
        {
        ( -64 -64 -16 ) ( -64 -63 -16 ) ( -64 -64 -15 ) __TB_empty 0 0 0 1 1
        ( -64 -64 -16 ) ( -64 -64 -15 ) ( -63 -64 -16 ) __TB_empty 0 0 0 1 1
        ( -64 -64 -16 ) ( -63 -64 -16 ) ( -64 -63 -16 ) __TB_empty 0 0 0 1 1
        ( 64 64 16 ) ( 64 65 16 ) ( 65 64 16 ) __TB_empty 0 0 0 1 1
        ( 64 64 16 ) ( 65 64 16 ) ( 64 64 17 ) __TB_empty 0 0 0 1 1
        ( 64 64 16 ) ( 64 64 17 ) ( 64 65 16 ) __TB_empty 0 0 0 1 1
        }
        }
    "#;

    #[test]
    fn from_str_test() {
        match Map::from_str(&String::default()) {
            Ok(map) => println!("{map:#?}"),
            Err(e) => panic!("{e}"),
        }

        match Map::from_str(MAP_EMPTY_ENTITY) {
            Ok(map) => println!("{map:#?}"),
            Err(e) => panic!("{e}"),
        }

        match Map::from_str(MAP_PROPERTY) {
            Ok(map) => println!("{map:#?}"),
            Err(e) => panic!("{e}"),
        }

        match Map::from_str(MAP_BRUSH) {
            Ok(map) => println!("{map:#?}"),
            Err(e) => panic!("{e}"),
        }

        match Map::from_str(MAP_SQUARE) {
            Ok(map) => println!("{map:#?}"),
            Err(e) => panic!("{e}"),
        }
    }
}
