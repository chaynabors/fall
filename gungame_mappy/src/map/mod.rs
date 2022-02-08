mod brush;
mod entity;
mod plane;

use std::fs;
use std::path::Path;
use std::str::FromStr;

use anyhow::Error;
use anyhow::Result;
use nom::error::VerboseError;

use entity::Entity;
use nom::error::convert_error;

#[derive(Debug, Default)]
pub struct Map {
    pub entities: Vec<Entity>
}

impl Map {
    pub fn load<P>(path: P) -> Result<Self> where P: AsRef<Path> {
        fs::read_to_string(path)?.parse()
    }
}

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        parser::parse_map::<VerboseError<&str>>(s)
            .map_err(|e| anyhow::anyhow!(convert_error(s, e)))
    }
}

mod parser {
    use nalgebra::Vector3;
    use nom::Finish;
    use nom::IResult;
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::bytes::complete::take_until;
    use nom::character::complete::char;
    use nom::character::complete::multispace1;
    use nom::character::complete::not_line_ending;
    use nom::combinator::map;
    use nom::combinator::value;
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

    enum Token<'a> {
        LBrace,
        RBrace,
        LParen,
        RParen,
        String(&'a str),
        Num(f32),
        Tex(&'a str),
    }

    fn comment<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
        preceded(tag("//"), not_line_ending)(i)
    }

    fn ws<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
        value((), many0(alt((multispace1, comment))))(i)
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

    fn point<'a, E: ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> IResult<&'a str, Vector3<f32>, E> {
        context(
            "point",
            delimited(
                pair(char('('), ws),
                map(tuple((num, ws, num, ws, num)), |(x, _, y, _, z)| Vector3::new(x, y, z)),
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
                tuple((ws, char('}'), ws)),
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

    pub fn parse_map<'a, E: std::error::Error + ParseError<&'a str> + ContextError<&'a str>>(i: &'a str) -> Result<Map, E> {
        context("parse_map",
            preceded(ws, map(many0(entity), |entities| Map { entities }))
        )(i).finish().map(|(_, map)| map)
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::Map;

    #[test]
    fn map_empty() {
        test_map(&String::default());
    }

    #[test]
    fn map_entity() {
        test_map(&"{}");
    }

    #[test]
    fn map_property() {
        test_map(&"{\"hello\" \"world\"}");
    }

    #[test]
    fn map_brush() {
        test_map(&"{{( -64 -64 -16 ) ( -64 -63 -16 ) ( -64 -64 -15 ) __TB_empty 0 0 0 1 1 }}");
    }

    #[test]
    fn map_cube() {
        test_map(&r#"
            // Game: Generic
            // Format: Standard
            // entity 0
            {
            "classname" "worldspawn"
            // brush 0
            {
            ( -64 -16 -64 ) ( -64 -16 -63 ) ( -64 -17 -64 ) __TB_empty 0 0 90 1 1
            ( 64 -64 64 ) ( 64 -64 65 ) ( 65 -64 64 ) __TB_empty 0 0 0 1 1
            ( -64 -16 -64 ) ( -64 -17 -64 ) ( -63 -16 -64 ) __TB_empty 0 0 0 1 -1
            ( 64 -48 64 ) ( 65 -48 64 ) ( 64 -49 64 ) __TB_empty 0 0 0 1 -1
            ( -64 64 -64 ) ( -63 64 -64 ) ( -64 64 -63 ) __TB_empty 0 0 0 1 1
            ( 64 -48 64 ) ( 64 -49 64 ) ( 64 -48 65 ) __TB_empty 0 0 90 1 1
            }
            }
        "#);
    }

    #[test]
    fn map_cabin() {
        test_map(include_str!("cabin.map"));
    }

    fn test_map(map: &str) {
        match Map::load(map) {
            Ok(map) => println!("{map:#?}"),
            Err(e) => panic!("{e}"),
        }
    }
}
