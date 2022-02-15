use std::collections::HashMap;

use anyhow::Result;
use nalgebra::Matrix3;
use nalgebra::Vector3;
use nom::Finish;
use nom::error::convert_error;
use nom::error::VerboseError;

#[derive(Clone, Debug, Default)]
pub struct Plane<'a> {
    pub points: [Vector3<f32>; 3],
    pub texture: &'a str,
    pub x_offset: f32,
    pub y_offset: f32,
    pub rotation: f32,
    pub x_scale: f32,
    pub y_scale: f32,
}

impl<'a> Plane<'a> {
    pub fn new(
        points: [Vector3<f32>; 3],
        texture: &'a str,
        x_offset: f32,
        y_offset: f32,
        rotation: f32,
        x_scale: f32,
        y_scale: f32
    ) -> Self {
        Self {
            points,
            texture,
            x_offset,
            y_offset,
            rotation,
            x_scale,
            y_scale,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Brush<'a> {
    planes: Vec<Plane<'a>>,
}

impl<'a> Brush<'a> {
    pub fn new(planes: Vec<Plane<'a>>) -> Self {
        Self { planes }
    }

    pub fn vertices(&self) -> Vec<Vector3<f32>> {
        let mut normals = vec![];
        for plane in &self.planes {
            let points = plane.points;
            normals.push((points[1] - points[0]).cross(&(points[2] - points[0])).normalize());
        }

        let mut vertices: Vec<Vector3<f32>> = vec![];
        for i in 0..self.planes.len() {
            let p1 = &self.planes[i];
            let n1 = &normals[i];
            for j in 0..self.planes.len() {
                let p2 = &self.planes[j];
                let n2 = &normals[j];
                if i == j || n1 == n2 { continue; }
                for k in 0..self.planes.len() {
                    let p3 = &self.planes[k];
                    let n3 = &normals[k];
                    if i == k || j == k || n1 == n3 || n2 == n3 { continue; }

                    let n = Matrix3::from_columns(&[normals[i], normals[j], normals[k]]);
                    let vertex: Vector3<f32> = 1.0 / n.determinant()
                        * (p1.points[0].dot(&n1) * n2.cross(&n3)
                        +  p2.points[0].dot(&n2) * n3.cross(&n1)
                        +  p3.points[0].dot(&n3) * n1.cross(&n2));
                    vertices.push(vertex);
                }
            }
        }

        for i in 0..self.planes.len() {
            vertices.retain(|vertex| (vertex - self.planes[i].points[0]).dot(&normals[i]) >= -0.2);
        }

        vertices
    }
}

#[derive(Clone, Debug, Default)]
pub struct Entity<'a> {
    pub properties: HashMap<&'a str, &'a str>,
    pub brushes: Vec<Brush<'a>>,
}

#[derive(Debug, Default)]
pub struct Map<'a> {
    pub entities: Vec<Entity<'a>>
}

impl<'a> Map<'a> {
    pub fn new(entities: Vec<Entity<'a>>) -> Self {
        Self { entities }
    }

    pub fn from_str(s: &'a str) -> Result<Self> {
        match map_parser::parse::<VerboseError<&str>>(s).finish() {
            Ok((_, map)) => Ok(map),
            Err(e) => Err(anyhow::anyhow!(convert_error(s, e))),
        }
    }
}

mod map_parser {
    use nalgebra::Vector3;
    use nom::IResult;
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::bytes::complete::take_until;
    use nom::bytes::complete::take_while;
    use nom::character::complete::anychar;
    use nom::character::complete::char;
    use nom::character::complete::multispace1;
    use nom::character::complete::not_line_ending;
    use nom::combinator::map;
    use nom::combinator::verify;
    use nom::error::ContextError;
    use nom::error::ParseError;
    use nom::multi::many0;
    use nom::number::complete::float;
    use nom::sequence::delimited;
    use nom::sequence::preceded;
    use nom::sequence::separated_pair;
    use nom::sequence::terminated;
    use nom::sequence::tuple;

    use super::Brush;
    use super::Entity;
    use super::Map;
    use super::Plane;

    fn comment<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        i: &'a str,
    ) -> IResult<&'a str, &'a str, E> {
        preceded(tag("//"), not_line_ending)(i)
    }

    fn ws<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        i: &'a str,
    ) -> IResult<&'a str, Vec<&'a str>, E> {
        many0(alt((multispace1, comment)))(i)
    }

    fn string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        i: &'a str
    ) -> IResult<&'a str, &'a str, E> {
        delimited(char('"'), take_until("\""), char('"'))(i)
    }

    fn property<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        i: &'a str
    ) -> IResult<&'a str, (&'a str, &'a str), E> {
        separated_pair(string, ws, string)(i)
    }

    fn point<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        i: &'a str
    ) -> IResult<&'a str, Vector3<f32>, E> {
        map(
            tuple((char('('), ws, float, ws, float, ws, float, ws, char(')'))),
            |(_, _, x, _, y, _, z, _, _)| Vector3::new(x, y, z)
        )(i)
    }

    fn literal<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        i: &'a str
    ) -> IResult<&'a str, &'a str, E> {
        preceded(
            verify(anychar, |c| c == &'_' || c.is_alphabetic()),
            take_while(|c: char| c == '_' || c.is_alphabetic())
        )(i)
    }

    fn plane<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        i: &'a str
    ) -> IResult<&'a str, Plane, E> {
        map(
            tuple((point, ws, point, ws, point, ws, literal, ws,
                float, ws, float, ws, float, ws, float, ws, float)),
            |(a, _, b, _, c, _, tex, _, x, _, y, _, r, _, sx, _, sy)| {
                Plane::new([a, b, c], tex, x, y, r, sx, sy)
            },
        )(i)
    }

    fn brush<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        i: &'a str
    ) -> IResult<&'a str, Brush, E> {
        delimited(
            terminated(char('{'), ws),
            map(many0(terminated(plane, ws)), |planes| Brush::new(planes)),
            char('}'),
        )(i)
    }

    fn entity<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        i: &'a str
    ) -> IResult<&'a str, Entity, E> {
        let mut entity = Entity::default();

        let (i, _) = delimited(
            terminated(char('{'), ws),
            many0(terminated(
                alt((
                    map(property, |(key, value)| { entity.properties.insert(key, value); }),
                    map(brush, |brush| entity.brushes.push(brush)),
                )),
                ws,
            )),
            char('}'),
        )(i)?;

        Ok((i, entity))
    }

    pub fn parse<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        i: &'a str
    ) -> IResult<&'a str, Map, E> {
        preceded(ws, map(many0(terminated(entity, ws)), |entities| Map::new(entities)))(i)
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
        match Map::from_str(map) {
            Ok(map) => println!("{map:?}"),
            Err(e) => panic!("{e}"),
        }
    }
}
