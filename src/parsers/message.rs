use crate::structures::errors::{Error, Error::ParseError};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage};
use serenity::utils::Colour;
use std::collections::HashMap;
use std::convert::TryFrom;

// TODO: PR to library and drop ASAP
pub static NAME_TO_COLOUR_MAPPING: Lazy<HashMap<&str, Colour>> = Lazy::new(|| {
    return maplit::hashmap! {
        "BLITZ_BLUE" => Colour::BLITZ_BLUE,
        "BLUE" => Colour::BLUE,
        "BLURPLE" => Colour::BLURPLE,
        "DARK_BLUE" => Colour::DARK_BLUE,
        "DARK_GOLD" => Colour::DARK_GOLD,
        "DARK_GREEN" => Colour::DARK_GREEN,
        "DARK_GREY" => Colour::DARK_GREY,
        "DARK_MAGENTA" => Colour::DARK_MAGENTA,
        "DARK_ORANGE" => Colour::DARK_ORANGE,
        "DARK_PURPLE" => Colour::DARK_PURPLE,
        "DARK_RED" => Colour::DARK_RED,
        "DARK_TEAL" => Colour::DARK_TEAL,
        "DARKER_GREY" => Colour::DARKER_GREY,
        "FABLED_PINK" => Colour::FABLED_PINK,
        "FADED_PURPLE" => Colour::FADED_PURPLE,
        "FOOYOO" => Colour::FOOYOO,
        "GOLD" => Colour::GOLD,
        "KERBAL" => Colour::KERBAL,
        "LIGHT_GREY" => Colour::LIGHT_GREY,
        "LIGHTER_GREY" => Colour::LIGHTER_GREY,
        "MAGENTA" => Colour::MAGENTA,
        "MEIBE_PINK" => Colour::MEIBE_PINK,
        "ORANGE" => Colour::ORANGE,
        "PURPLE" => Colour::PURPLE,
        "RED" => Colour::RED,
        "ROHRKATZE_BLUE" => Colour::ROHRKATZE_BLUE,
        "ROSEWATER" => Colour::ROSEWATER,
        "TEAL" => Colour::TEAL,
    };
});

#[derive(Deserialize, Debug, PartialEq)]
struct Message {
    #[serde(alias = "c")]
    content: Option<String>,
    #[serde(alias = "e")]
    embed: Option<Embed>,
}

impl<'a> TryFrom<Message> for CreateMessage<'a> {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        let mut builder = Self::default();
        message.content.map(|c| builder.content(c));
        if let Some(e) = message.embed {
            let embed = CreateEmbed::try_from(e)?;
            builder.embed(|e| {
                e.0 = embed.0;
                e
            });
        }
        return Ok(builder);
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct Embed {
    #[serde(alias = "c")]
    #[serde(alias = "color")]
    colour: Option<EmbedColourEnum>,
    #[serde(alias = "d")]
    description: Option<String>,
    #[serde(alias = "f")]
    #[serde(alias = "fields")]
    field: Option<EmbedFieldEnum>,
    footer: Option<EmbedFooterEnum>,
    #[serde(alias = "a")]
    author: Option<EmbedAuthor>,
}

impl TryFrom<Embed> for CreateEmbed {
    type Error = Error;

    fn try_from(value: Embed) -> Result<Self, Self::Error> {
        let mut builder = CreateEmbed::default();
        match value.colour {
            None => Result::<(), Self::Error>::Ok(()),
            Some(v) => {
                let colour = Colour::try_from(v)?;
                builder.colour(colour);
                Ok(())
            }
        }?;
        value.description.map(|v| builder.description(v));
        value.field.map(|v| {
            let fields = match v {
                EmbedFieldEnum::Single(field) => vec![field],
                EmbedFieldEnum::Vector(fields) => fields,
            };
            for field in fields {
                builder.field(field.name, field.value, field.inline.unwrap_or(false));
            }
        });
        value.footer.map(|v| {
            builder.footer(|f| {
                f.0 = CreateEmbedFooter::from(v).0;
                f
            })
        });
        value.author.map(|v| {
            builder.author(|a| {
                a.0 = CreateEmbedAuthor::from(v).0;
                a
            })
        });
        return Ok(builder);
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum EmbedColourEnum {
    Integer(u32),
    String(String),
    RGB(RGBColour),
}

impl TryFrom<EmbedColourEnum> for Colour {
    type Error = Error;

    fn try_from(embed_enum: EmbedColourEnum) -> Result<Self, Self::Error> {
        match embed_enum {
            EmbedColourEnum::Integer(int) => Ok(Colour(int)),
            EmbedColourEnum::String(string) => {
                return if string.starts_with("#") {
                    if string.len() == 7 {
                        let decoded = u32::from_str_radix(&string[1..], 16)?;
                        Ok(Colour::from(decoded))
                    } else {
                        Err(ParseError(format!(
                            "String {:?} is not a valid hex color",
                            string
                        )))
                    }
                } else {
                    NAME_TO_COLOUR_MAPPING
                        .get(string.as_str())
                        .ok_or(ParseError(format!(
                            "String {:?} is neither a hex, not a valid color name.",
                            string
                        )))
                        .map(|c| c.clone())
                }
            }
            EmbedColourEnum::RGB(rgb) => Ok(Colour::from_rgb(rgb.red, rgb.green, rgb.blue)),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct RGBColour {
    #[serde(alias = "r")]
    red: u8,
    #[serde(alias = "g")]
    green: u8,
    #[serde(alias = "b")]
    blue: u8,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum EmbedFieldEnum {
    Single(EmbedField),
    Vector(Vec<EmbedField>),
}
#[derive(Deserialize, Debug, PartialEq)]
struct EmbedField {
    name: String,
    value: String,
    inline: Option<bool>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum EmbedFooterEnum {
    TextOnly(String),
    Complex(EmbedFooter),
}

impl From<EmbedFooterEnum> for EmbedFooter {
    fn from(ef: EmbedFooterEnum) -> Self {
        match ef {
            EmbedFooterEnum::TextOnly(text) => EmbedFooter {
                text: Some(text),
                url: None,
            },
            EmbedFooterEnum::Complex(footer) => footer,
        }
    }
}

impl From<EmbedFooterEnum> for CreateEmbedFooter {
    fn from(ef: EmbedFooterEnum) -> Self {
        let footer = EmbedFooter::from(ef);
        CreateEmbedFooter::from(footer)
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct EmbedFooter {
    text: Option<String>,
    url: Option<String>,
}

impl From<EmbedFooter> for CreateEmbedFooter {
    fn from(ef: EmbedFooter) -> Self {
        let mut builder = Self::default();
        ef.text.map(|v| builder.text(v));
        ef.url.map(|v| builder.icon_url(v));
        return builder;
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct EmbedAuthor {
    #[serde(alias = "n")]
    name: String,
    #[serde(alias = "u")]
    #[serde(alias = "url")]
    #[serde(alias = "l")]
    link: Option<String>,
    #[serde(alias = "i")]
    icon: Option<String>,
}

impl From<EmbedAuthor> for CreateEmbedAuthor {
    fn from(a: EmbedAuthor) -> Self {
        let mut builder = Self::default();
        builder.name(a.name);
        a.icon.map(|v| builder.icon_url(v));
        a.link.map(|v| builder.url(v));
        return builder;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use proptest::prelude::*;
    use rstest::rstest;

    #[test]
    fn content_only_deserialization() {
        let input = r#"{"content": "My Important Message"}"#;
        let deserialized: Message = serde_json::from_str(input).unwrap();
        let expected = Message {
            content: Some("My Important Message".to_string()),
            embed: None,
        };
        assert_eq!(expected, deserialized);
    }

    #[test]
    fn deserializing_embed_filed() {
        let input = r#"{"name": "Title", "value": "My Val"}"#;
        let deserialized: EmbedField = serde_json::from_str(input).unwrap();
        let expected = EmbedField {
            name: "Title".to_string(),
            value: "My Val".to_string(),
            inline: None,
        };
        assert_eq!(expected, deserialized);
    }

    #[test]
    fn message_with_simple_embed() {
        let input = r#"{"e": {"d": "My Description"}}"#;
        let deserialized: Message = serde_json::from_str(input).unwrap();
        let expected = Message {
            content: None,
            embed: Some(Embed {
                colour: None,
                description: Some("My Description".to_string()),
                field: None,
                footer: None,
                author: None,
            }),
        };
        assert_eq!(expected, deserialized);
    }

    #[test]
    fn colour_embed() {
        let input = r#"{"colour": "RED"}"#;
        let deserialized: Embed = serde_json::from_str(input).unwrap();
        let expected = Embed {
            colour: Some(EmbedColourEnum::String("RED".to_string())),
            description: None,
            field: None,
            footer: None,
            author: None,
        };
        assert_eq!(expected, deserialized);
    }

    #[test]
    fn complex_message() {
        let input = r#"
            {
                "content": "Content",
                "embed": {
                    "colour": "RED",
                    "description": "Description",
                    "fields": [
                        {
                            "name": "Name",
                            "value": "Value",
                            "inline": true
                        }
                    ],
                    "footer": "Footer",
                    "author": {
                        "name": "Name"
                     }
                }
            }
        "#;
        let deserialized: Message = serde_json::from_str(input).unwrap();
        let expected = Message {
            content: Some("Content".to_string()),
            embed: Some(Embed {
                colour: Some(EmbedColourEnum::String("RED".to_string())),
                description: Some("Description".to_string()),
                field: Some(EmbedFieldEnum::Vector(vec![EmbedField {
                    name: "Name".to_string(),
                    value: "Value".to_string(),
                    inline: Some(true),
                }])),
                footer: Some(EmbedFooterEnum::TextOnly("Footer".to_string())),
                author: Some(EmbedAuthor {
                    name: "Name".to_string(),
                    link: None,
                    icon: None,
                }),
            }),
        };
        assert_eq!(expected, deserialized);
    }

    #[test]
    fn complex_cast() {
        let input = Message {
            content: Some("Content".to_string()),
            embed: Some(Embed {
                colour: Some(EmbedColourEnum::String("RED".to_string())),
                description: Some("Description".to_string()),
                field: Some(EmbedFieldEnum::Vector(vec![EmbedField {
                    name: "Name".to_string(),
                    value: "Value".to_string(),
                    inline: Some(true),
                }])),
                footer: Some(EmbedFooterEnum::TextOnly("Footer".to_string())),
                author: Some(EmbedAuthor {
                    name: "Name".to_string(),
                    link: None,
                    icon: None,
                }),
            }),
        };
        let mut expected = CreateMessage::default();
        expected.content("Content").embed(|e| {
            e.colour(Colour::RED)
                .description("Description")
                .field("Name", "Value", true)
                .footer(|f| f.text("Footer"))
                .author(|a| a.name("Name"))
        });
        let result = CreateMessage::try_from(input);
        assert!(matches!(result, Ok(_)));
        assert_eq!(expected.0, result.unwrap().0); // Only compare HashMap
    }

    #[rstest(
        hex,
        case::sixteen_to_sixth_power("#1000000"),
        case::one_to_eight("#12345678")
    )]
    fn invalid_hex(hex: &str) {
        let result = Colour::try_from(EmbedColourEnum::String(hex.to_string()));
        assert!(matches!(result, Err(ParseError(_))));
    }

    #[rstest(
        hex,
        value,
        case::zero("#000000", 0),
        case::one("#000001", 1),
        case::sixteen("#000010", 16),
        case::complex("#234099", 2310297)
    )]
    fn hex_parsing(hex: &str, value: u32) {
        let c = Colour::try_from(EmbedColourEnum::String(hex.to_string()));
        assert_eq!(Ok(Colour(value)), c);
    }

    proptest! {
        #[test]
        fn parses_all_valid_hexes(expected in 0u32..(16_u32.pow(6))) {
            let colour = Colour(expected);
            let hex = format!("#{}", colour.hex());
            let result = Colour::try_from(EmbedColourEnum::String(hex));
            prop_assert_eq!(Ok(colour), result);
        }
    }
}
