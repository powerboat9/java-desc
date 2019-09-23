use nom::{named, terminated, is_not, char,
          switch, take, value, map,
          pair, fold_many_m_n, delimited, many0,
          alt, combinator::complete};

#[cfg(test)]
mod tests {
    use crate::{FieldType, SingleType, MethodType};

    #[test]
    fn fields() {
        assert_eq!(FieldType::parse("[[[Lfoo bar net;"), Some(FieldType {
            base: SingleType::Reference(String::from("foo bar net")), array_cnt: 3
        }));
    }
    #[test]
    fn methods() {
        assert_eq!(MethodType::parse("([B[[LFoo;I)[LNetwork;"), Some(MethodType {
            params: vec![FieldType {
                base: SingleType::Byte,
                array_cnt: 1
            }, FieldType {
                base: SingleType::Reference(String::from("Foo")),
                array_cnt: 2
            }, FieldType {
                base: SingleType::Int,
                array_cnt: 0
            }],
            ret: FieldType {
                base: SingleType::Reference(String::from("Network")),
                array_cnt: 1
            }
        }))
    }
}

named!(semi_terminated<&str, &str>, terminated!(is_not!(";"), char!(';')));

named!(single<&str, SingleType>, switch!(take!(1),
    "B" => value!(SingleType::Byte) |
    "C" => value!(SingleType::Char) |
    "D" => value!(SingleType::Double) |
    "F" => value!(SingleType::Float) |
    "I" => value!(SingleType::Int) |
    "J" => value!(SingleType::Long) |
    "S" => value!(SingleType::Short) |
    "Z" => value!(SingleType::Boolean) |
    "L" => map!(semi_terminated, |v| SingleType::Reference(String::from(v))))
);

named!(field<&str, FieldType>, map!(
    pair!(fold_many_m_n!(0, 255, char!('['), 0u8, |v, _| v + 1), single),
    |v: (u8, SingleType)| FieldType {base: v.1, array_cnt: v.0}
));

named!(method<&str, MethodType>, map!(pair!(
    delimited!(char!('('), many0!(field), char!(')')),
    field),
    |v| MethodType {params: v.0, ret: v.1}
));

named!(descriptor<&str, Descriptor>, alt!(
    map!(method, |v| Descriptor::Method(v)) |
    map!(field, |v| Descriptor::Field(v))
));

/// This is an enum, used to represent a base type (without array data)
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SingleType {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Reference(String),
    Short,
    Boolean
}

impl SingleType {
    pub fn parse(input: &str) -> Option<Self> {
        complete(single)(input).ok().map(|v| v.1)
    }
}

/// This is a type representing a type descriptor
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FieldType {
    pub base: SingleType,
    pub array_cnt: u8
}

impl FieldType {
    pub fn parse(input: &str) -> Option<Self> {
        complete(field)(input).ok().map(|v| v.1)
    }
}

impl From<SingleType> for FieldType {
    fn from(v: SingleType) -> Self {
        Self {
            base: v,
            array_cnt: 0
        }
    }
}

/// This is a type representing a method descriptor
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MethodType {
    pub params: Vec<FieldType>,
    pub ret: FieldType
}

impl MethodType {
    pub fn parse(input: &str) -> Option<Self> {
        complete(method)(input).ok().map(|v| v.1)
    }
}

/// This is a type representing either a field or method descriptor
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Descriptor {
    Field(FieldType),
    Method(MethodType)
}

impl Descriptor {
    pub fn parse(input: &str) -> Option<Self> {
        complete(descriptor)(input).ok().map(|v| v.1)
    }
}

impl From<FieldType> for Descriptor {
    /// Creates a Signature from a FieldType
    fn from(v: FieldType) -> Self {
        Descriptor::Field(v)
    }
}

impl From<MethodType> for Descriptor {
    /// Creates a Signature from a MethodType
    fn from(v: MethodType) -> Self {
        Descriptor::Method(v)
    }
}