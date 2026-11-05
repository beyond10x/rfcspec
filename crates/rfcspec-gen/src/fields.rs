//! The multi-field path: a header of unsigned big-endian fields of 1 to 64 bits, packed most
//! significant bit first, a field present only when an earlier field has a value, with `equals`,
//! `range` and `one_of` constraints, and the length of what follows read from the first present
//! field the layout's `length` names.
//!
//! The generated crate has a struct named after the layout, `parse` (the header and the octets it
//! used; too few octets is `Error::Incomplete`, a violated constraint `Error::Constraint` naming
//! the field), `print`, and `length()` when the layout names its length. Its tests are computed
//! here, by a decoder independent of the generated one:
//!
//! - one per octet example of the RFC: the parsed header field by field, the octets it used,
//!   printing gives the header octets back, one octet fewer is incomplete, and the octets after the
//!   header plus the elided ones are the length;
//! - the layout vectors of design section 11, built on the header of the first example: each
//!   range at its minimum and maximum (accepted), the two values below its minimum and the one
//!   above its maximum that the field's width holds (refused), each `equals` value (accepted) and
//!   the values one above and one below it that the field's width holds (refused), and each
//!   presence selector at the value below the smallest value that selects a field (accepted). A
//!   layout without an octet example builds them on the header whose present fields are all 0.

use std::collections::BTreeSet;

use anyhow::{Result, bail, ensure};
use rfcspec_model::{Catalogue, Constraint, Field, Layout, OctetExample, Overlay};

use super::camel;

/// Field values by index; `None` for an absent field.
type Values = Vec<Option<u64>>;

/// Why octets are not a header, as the generated crate reports it.
#[derive(Debug, PartialEq, Eq)]
enum Decoded {
    Incomplete,
    Constraint(usize, u64),
}

/// Validate a layout for the multi-field path.
fn validate(layout: &Layout) -> Result<()> {
    ensure!(
        !layout.fields.is_empty(),
        "layout {}: no fields",
        layout.name
    );
    let mut earlier = BTreeSet::new();
    for field in &layout.fields {
        ensure!(
            (1..=64).contains(&field.bits),
            "layout {}: field {} has {} bits; 1 to 64 are supported",
            layout.name,
            field.name,
            field.bits
        );
        ensure!(
            !field.signed && field.order == "big-endian",
            "layout {}: field {} must be unsigned and big-endian",
            layout.name,
            field.name
        );
        ensure!(
            ident(&field.name).is_some(),
            "layout {}: field {} is no Rust identifier once dashes become underscores",
            layout.name,
            field.name
        );
        if let Some(presence) = &field.presence {
            ensure!(
                earlier.contains(presence.field.as_str()),
                "layout {}: field {} is present when {}, which is no earlier field",
                layout.name,
                field.name,
                presence.field
            );
        }
        let (low, high) = field.bounds();
        for constraint in &field.constraints {
            for value in constraint.values() {
                ensure!(
                    (low..=high).contains(&i128::from(value)),
                    "layout {}: field {} constraint value {value} does not fit {} bits",
                    layout.name,
                    field.name,
                    field.bits
                );
            }
        }
        earlier.insert(field.name.as_str());
    }
    for name in &layout.length {
        ensure!(
            layout.fields.iter().any(|f| &f.name == name),
            "layout {}: length names {name}, which is no field",
            layout.name
        );
    }
    Ok(())
}

fn ident(name: &str) -> Option<String> {
    let ident = name.replace('-', "_");
    let valid = ident.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
        && ident
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !matches!(
            ident.as_str(),
            "type" | "fn" | "let" | "match" | "mod" | "move" | "ref" | "self" | "struct" | "use"
        );
    valid.then_some(ident)
}

fn rust_type(bits: u32) -> &'static str {
    match bits {
        1..=8 => "u8",
        9..=16 => "u16",
        17..=32 => "u32",
        _ => "u64",
    }
}

/// `u64::from(expr)`, or `expr` itself for a 64-bit field.
fn widen(expr: &str, bits: u32) -> String {
    if bits > 32 {
        expr.to_owned()
    } else {
        format!("u64::from({expr})")
    }
}

fn index_of(layout: &Layout, name: &str) -> usize {
    layout
        .fields
        .iter()
        .position(|f| f.name == name)
        .expect("validated")
}

fn present(layout: &Layout, values: &Values, field: &Field) -> bool {
    field.presence.as_ref().is_none_or(|presence| {
        u64::try_from(presence.equals)
            .is_ok_and(|equals| values[index_of(layout, &presence.field)] == Some(equals))
    })
}

fn admits(field: &Field, value: u64) -> bool {
    field
        .constraints
        .iter()
        .all(|constraint| constraint.admits(i128::from(value)))
}

fn decode(layout: &Layout, bytes: &[u8]) -> Result<(Values, usize), Decoded> {
    let mut values: Values = vec![None; layout.fields.len()];
    let mut bit = 0usize;
    for (index, field) in layout.fields.iter().enumerate() {
        if !present(layout, &values, field) {
            continue;
        }
        let bits = field.bits as usize;
        if bit + bits > bytes.len() * 8 {
            return Err(Decoded::Incomplete);
        }
        let mut value = 0u64;
        for _ in 0..bits {
            value = (value << 1) | u64::from((bytes[bit / 8] >> (7 - bit % 8)) & 1);
            bit += 1;
        }
        if !admits(field, value) {
            return Err(Decoded::Constraint(index, value));
        }
        values[index] = Some(value);
    }
    Ok((values, bit.div_ceil(8)))
}

fn encode(layout: &Layout, values: &Values) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut bit = 0usize;
    for (index, field) in layout.fields.iter().enumerate() {
        if !present(layout, values, field) {
            continue;
        }
        let value = values[index].unwrap_or(0);
        for shift in (0..field.bits).rev() {
            if bit.is_multiple_of(8) {
                bytes.push(0);
            }
            let set = u8::from((value >> shift) & 1 == 1);
            *bytes.last_mut().expect("pushed") |= set << (7 - bit % 8);
            bit += 1;
        }
    }
    bytes
}

/// The values with presence settled in order: a present field without a value gets 0, an absent
/// field loses its value.
fn settle(layout: &Layout, mut values: Values) -> Values {
    for (index, field) in layout.fields.iter().enumerate() {
        if present(layout, &values, field) {
            values[index].get_or_insert(0);
        } else {
            values[index] = None;
        }
    }
    values
}

/// `base` with `field` set to `value` and every selector it needs set so that it is present.
fn with(layout: &Layout, base: &Values, field: usize, value: u64) -> Values {
    let mut values = base.clone();
    let mut current = &layout.fields[field];
    while let Some(presence) = &current.presence {
        let selector = index_of(layout, &presence.field);
        values[selector] = u64::try_from(presence.equals).ok();
        current = &layout.fields[selector];
    }
    values[field] = Some(value);
    settle(layout, values)
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn literal(layout: &Layout, name: &str, values: &Values) -> String {
    let fields: Vec<String> = layout
        .fields
        .iter()
        .zip(values)
        .map(|(field, value)| {
            let ident = ident(&field.name).expect("validated");
            match (&field.presence, value) {
                (None, Some(v)) => format!("{ident}: {v}"),
                (Some(_), Some(v)) => format!("{ident}: Some({v})"),
                (_, None) => format!("{ident}: None"),
            }
        })
        .collect();
    format!(
        "{name} {{\n                {}\n            }}",
        fields.join(",\n                ")
    )
}

fn length_of(layout: &Layout, values: &Values) -> Option<u64> {
    layout
        .length
        .iter()
        .find_map(|name| values[index_of(layout, name)])
}

/// One layout vector: the field, its value, the octets, and whether they are accepted.
struct Vector {
    field: usize,
    value: u64,
    octets: Vec<u8>,
    accepted: bool,
}

fn vectors(layout: &Layout, base: &Values) -> Result<Vec<Vector>> {
    let mut found: Vec<Vector> = Vec::new();
    let mut push = |field: usize, value: u64, accepted: bool| -> Result<()> {
        if found.iter().any(|v| v.field == field && v.value == value) {
            return Ok(());
        }
        let values = with(layout, base, field, value);
        let octets = encode(layout, &values);
        let name = &layout.fields[field].name;
        match (decode(layout, &octets), accepted) {
            (Ok((decoded, used)), true) if decoded == values && used == octets.len() => {}
            (Err(Decoded::Constraint(at, refused)), false) if at == field && refused == value => {}
            (outcome, _) => bail!(
                "layout {}: the vector {name} = {value} does not hold as built ({outcome:?})",
                layout.name
            ),
        }
        found.push(Vector {
            field,
            value,
            octets,
            accepted,
        });
        Ok(())
    };
    for (index, field) in layout.fields.iter().enumerate() {
        let (_, width) = field.bounds();
        for constraint in &field.constraints {
            if let Constraint::Equals(value) = constraint {
                let value = i128::from(*value);
                push(index, u64::try_from(value)?, true)?;
                for neighbour in [value + 1, value - 1] {
                    if (0..=width).contains(&neighbour) {
                        push(index, u64::try_from(neighbour)?, false)?;
                    }
                }
                continue;
            }
            let Constraint::Range([low, high]) = constraint else {
                continue;
            };
            let (low, high) = (i128::from(*low), i128::from(*high));
            push(index, u64::try_from(low)?, true)?;
            push(index, u64::try_from(high)?, true)?;
            for below in [low - 2, low - 1] {
                if below >= 0 {
                    push(index, u64::try_from(below)?, false)?;
                }
            }
            if high < width {
                push(index, u64::try_from(high + 1)?, false)?;
            }
        }
    }
    for (index, field) in layout.fields.iter().enumerate() {
        let selecting: BTreeSet<i64> = layout
            .fields
            .iter()
            .filter_map(|f| f.presence.as_ref())
            .filter(|p| p.field == field.name)
            .map(|p| p.equals)
            .collect();
        if let Some(&smallest) = selecting.first()
            && smallest > 0
            && !selecting.contains(&(smallest - 1))
            && admits(field, u64::try_from(smallest - 1)?)
        {
            push(index, u64::try_from(smallest - 1)?, true)?;
        }
    }
    Ok(found)
}

/// The library of a multi-field layout.
pub fn library(
    overlay: &Overlay,
    catalogue: &Catalogue,
    profile: &str,
    layout: &Layout,
) -> Result<String> {
    validate(layout)?;
    let name = camel(&layout.name);

    let mut struct_fields = String::new();
    let mut parse_fields = String::new();
    let mut print_fields = String::new();
    let mut idents = Vec::new();
    let mut admit_fns = String::new();
    for field in &layout.fields {
        let ident = ident(&field.name).expect("validated");
        let ty = rust_type(field.bits);
        let mut doc = format!(
            "`{}`, {} bit{}",
            field.name,
            field.bits,
            if field.bits == 1 { "" } else { "s" }
        );
        let condition = presence_condition(layout, field, "");
        if let Some(presence) = &field.presence {
            doc.push_str(&format!(
                ", present when `{}` is {}",
                presence.field, presence.equals
            ));
        }
        let mut admits = Vec::new();
        for constraint in &field.constraints {
            match constraint {
                Constraint::Equals(value) => {
                    doc.push_str(&format!(", equal to {value}"));
                    admits.push(format!("value == {value}"));
                }
                Constraint::Range([low, high]) => {
                    doc.push_str(&format!(", {low} to {high}"));
                    admits.push(format!("({low}..={high}).contains(&value)"));
                }
                Constraint::OneOf(values) => {
                    let list = values
                        .iter()
                        .map(i64::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    doc.push_str(&format!(", one of {list}"));
                    admits.push(format!("[{list}].contains(&value)"));
                }
            }
        }
        let check = if admits.is_empty() {
            "any".to_owned()
        } else {
            admit_fns.push_str(&format!(
                "\n/// Whether a value meets the constraints of `{}`.\nfn {ident}_admits(value: u64) -> bool {{\n    {}\n}}\n",
                field.name,
                admits.join(" && ")
            ));
            format!("{ident}_admits")
        };
        let args = format!("\"{}\", {}, {check}", field.name, field.bits);
        let cast = if field.bits > 32 {
            String::new()
        } else {
            format!(" as {ty}")
        };
        let read = |indent: usize| {
            if args.len() > 60 {
                let pad = " ".repeat(indent);
                format!(
                    "reader.field(\n{pad}    \"{}\",\n{pad}    {},\n{pad}    {check},\n{pad})?{cast}",
                    field.name, field.bits
                )
            } else {
                format!("reader.field({args})?{cast}")
            }
        };
        let value = if field.presence.is_some() {
            if field.bits > 32 {
                format!("header.{ident}.unwrap_or(0)")
            } else {
                format!("header.{ident}.map_or(0, u64::from)")
            }
        } else {
            widen(&format!("header.{ident}"), field.bits)
        };
        match &condition {
            Some(condition) => {
                struct_fields
                    .push_str(&format!("    /// {doc}.\n    pub {ident}: Option<{ty}>,\n"));
                parse_fields.push_str(&format!(
                    "    let {ident} = if {condition} {{\n        Some({})\n    }} else {{\n        None\n    }};\n",
                    read(8)
                ));
                let header_condition =
                    presence_condition(layout, field, "header.").expect("a present field");
                print_fields.push_str(&format!(
                    "    if {header_condition} {{\n        writer.field({}, {value});\n    }}\n",
                    field.bits
                ));
            }
            None => {
                struct_fields.push_str(&format!("    /// {doc}.\n    pub {ident}: {ty},\n"));
                parse_fields.push_str(&format!("    let {ident} = {};\n", read(4)));
                print_fields.push_str(&format!("    writer.field({}, {value});\n", field.bits));
            }
        }
        idents.push(ident);
    }

    let length = if layout.length.is_empty() {
        String::new()
    } else {
        let mut chain = String::new();
        for (position, field_name) in layout.length.iter().enumerate() {
            let field = &layout.fields[index_of(layout, field_name)];
            let ident = ident_of(field);
            let expr = if field.presence.is_some() {
                widen_option(&format!("self.{ident}"), field.bits)
            } else {
                format!("Some({})", widen(&format!("self.{ident}"), field.bits))
            };
            if position == 0 {
                chain.push_str(&expr);
            } else {
                chain.push_str(&format!("\n            .or({expr})"));
            }
        }
        format!(
            r#"
impl {name} {{
    /// The length of what follows the header: the first present of {list}.
    pub fn length(&self) -> u64 {{
        {chain}
            .unwrap_or(0)
    }}
}}
"#,
            list = layout
                .length
                .iter()
                .map(|n| format!("`{n}`"))
                .collect::<Vec<_>>()
                .join(", "),
        )
    };

    let examples: Vec<&OctetExample> = catalogue.octet_examples.iter().collect();
    let mut tests = String::new();
    let mut base: Option<Values> = None;
    for example in &examples {
        if let Some(finding) = overlay.findings.iter().find(|f| f.item == example.id) {
            tests.push_str(&format!(
                "\n    // {}: no test, review finding {}.\n",
                example.id, finding.id
            ));
            continue;
        }
        let (values, used) = match decode(layout, &example.octets) {
            Ok(decoded) => decoded,
            Err(error) => bail!(
                "example {} ({}) is no {} ({error:?}) and no review finding names it",
                example.id,
                example.text,
                layout.name
            ),
        };
        base.get_or_insert_with(|| values.clone());
        let mut body = format!(
            r#"
    /// {text} (RFC {rfc} lines {start}-{end}).
    #[test]
    fn example_{id}() {{
        let bytes = octets("{octets}");
        let (header, used) = parse(&bytes).unwrap();
        assert_eq!(
            header,
            {literal}
        );
        assert_eq!(used, {used});
        assert_eq!(print(&header), bytes[..used]);
        assert_eq!(parse(&bytes[..used - 1]), Err(Error::Incomplete));
"#,
            text = example.text.replace('`', "'"),
            rfc = overlay.rfc,
            start = example.span.start,
            end = example.span.end,
            id = example.id.to_lowercase(),
            octets = hex(&example.octets),
            literal = literal(layout, &name, &values),
        );
        if let Some(length) = length_of(layout, &values) {
            let elided = if example.elided == 0 {
                String::new()
            } else {
                format!(" + {}", example.elided)
            };
            body.push_str(&format!(
                "        assert_eq!(header.length(), {length});\n        assert_eq!((bytes.len() - used) as u64{elided}, header.length());\n"
            ));
        }
        body.push_str("    }\n");
        tests.push_str(&body);
    }
    let base = base.unwrap_or_else(|| settle(layout, vec![None; layout.fields.len()]));
    for vector in vectors(layout, &base)? {
        let field = &layout.fields[vector.field];
        let ident = ident_of(field);
        let verdict = if vector.accepted {
            "accepted"
        } else {
            "refused"
        };
        let body = if vector.accepted {
            let expected = if field.presence.is_some() {
                format!("Some({})", vector.value)
            } else {
                vector.value.to_string()
            };
            format!(
                "        let (header, used) = parse(&bytes).unwrap();\n        assert_eq!(header.{ident}, {expected});\n        assert_eq!(used, bytes.len());\n        assert_eq!(print(&header), bytes);\n"
            )
        } else {
            format!(
                "        assert_eq!(\n            parse(&bytes),\n            Err(Error::Constraint {{\n                field: \"{}\",\n                value: {}\n            }})\n        );\n",
                field.name, vector.value
            )
        };
        tests.push_str(&format!(
            "\n    #[test]\n    fn vector_{ident}_{value}_{verdict}() {{\n        let bytes = octets(\"{octets}\");\n{body}    }}\n",
            value = vector.value,
            octets = hex(&vector.octets),
        ));
    }

    let field_list = layout
        .fields
        .iter()
        .map(|f| format!("`{}` {}", f.name, f.bits))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!(
        r#"#![forbid(unsafe_code)]
//! Generated by `rfcspec generate rust` from the `{profile}` profile's overlay; do not edit.
//!
//! RFC {rfc} layout `{layout}`: unsigned big-endian fields packed most significant bit first,
//! with their widths in bits: {field_list}.

use std::fmt;

/// The `{layout}` layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct {name} {{
{struct_fields}}}
{length}
/// Why octets are not a `{layout}`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {{
    /// Fewer octets than the layout needs.
    Incomplete,
    /// A field's value violates its constraint.
    Constraint {{ field: &'static str, value: u64 }},
}}

impl fmt::Display for Error {{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
        match self {{
            Error::Incomplete => write!(f, "too few octets for a {layout}"),
            Error::Constraint {{ field, value }} => {{
                write!(f, "{{field}} = {{value}} violates its constraint")
            }}
        }}
    }}
}}

impl std::error::Error for Error {{}}

struct Reader<'a> {{
    bytes: &'a [u8],
    bit: usize,
}}

impl Reader<'_> {{
    fn field(
        &mut self,
        name: &'static str,
        bits: usize,
        admits: fn(u64) -> bool,
    ) -> Result<u64, Error> {{
        if self.bit + bits > self.bytes.len() * 8 {{
            return Err(Error::Incomplete);
        }}
        let mut value = 0u64;
        for _ in 0..bits {{
            let octet = self.bytes[self.bit / 8];
            value = (value << 1) | u64::from((octet >> (7 - self.bit % 8)) & 1);
            self.bit += 1;
        }}
        if admits(value) {{
            Ok(value)
        }} else {{
            Err(Error::Constraint {{ field: name, value }})
        }}
    }}
}}

#[derive(Default)]
struct Writer {{
    bytes: Vec<u8>,
    bit: usize,
}}

impl Writer {{
    fn field(&mut self, bits: u32, value: u64) {{
        for shift in (0..bits).rev() {{
            if self.bit.is_multiple_of(8) {{
                self.bytes.push(0);
            }}
            let set = u8::from((value >> shift) & 1 == 1);
            if let Some(last) = self.bytes.last_mut() {{
                *last |= set << (7 - self.bit % 8);
            }}
            self.bit += 1;
        }}
    }}
}}

/// Every value meets a field without constraints.
fn any(_: u64) -> bool {{
    true
}}
{admit_fns}
/// Parse a `{layout}` from the front of `bytes`: the header and the number of octets it used.
pub fn parse(bytes: &[u8]) -> Result<({name}, usize), Error> {{
    let mut reader = Reader {{ bytes, bit: 0 }};
{parse_fields}    Ok((
        {name} {{
            {idents}
        }},
        reader.bit.div_ceil(8),
    ))
}}

/// The octets of a `{layout}`; a field is written when its presence condition holds.
pub fn print(header: &{name}) -> Vec<u8> {{
    let mut writer = Writer::default();
{print_fields}    writer.bytes
}}

#[cfg(test)]
mod tests {{
    use super::*;

    /// Octets written as two hex digits each, separated by spaces.
    fn octets(hex: &str) -> Vec<u8> {{
        hex.split_whitespace()
            .map(|pair| u8::from_str_radix(pair, 16).unwrap())
            .collect()
    }}

    #[test]
    fn no_octets_are_incomplete() {{
        assert_eq!(parse(&[]), Err(Error::Incomplete));
    }}
{tests}}}
"#,
        rfc = overlay.rfc,
        layout = layout.name,
        idents = idents.join(",\n            ") + ",",
    ))
}

fn ident_of(field: &Field) -> String {
    ident(&field.name).expect("validated")
}

/// `expr`, an `Option` of the field's type, as an `Option<u64>`.
fn widen_option(expr: &str, bits: u32) -> String {
    if bits > 32 {
        expr.to_owned()
    } else {
        format!("{expr}.map(u64::from)")
    }
}

/// The presence condition of a field over `<prefix><selector>`; none for a field always present.
fn presence_condition(layout: &Layout, field: &Field, prefix: &str) -> Option<String> {
    let presence = field.presence.as_ref()?;
    let selector = &layout.fields[index_of(layout, &presence.field)];
    let expr = format!("{prefix}{}", ident_of(selector));
    Some(if selector.presence.is_some() {
        format!(
            "{} == Some({})",
            widen_option(&expr, selector.bits),
            presence.equals
        )
    } else {
        format!("{} == {}", widen(&expr, selector.bits), presence.equals)
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rfcspec_model::read_yaml;

    use super::*;

    fn layout(profile: &str) -> Layout {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../profiles")
            .join(profile)
            .join("overlay.yaml");
        let overlay: Overlay = read_yaml(&path).unwrap();
        let [layout] = overlay.layouts.as_slice() else {
            panic!("{profile}: {} layouts", overlay.layouts.len());
        };
        layout.clone()
    }

    /// Each vector as field, value, octets and verdict.
    fn listed(layout: &Layout, base: &Values) -> Vec<(String, u64, String, &'static str)> {
        vectors(layout, base)
            .unwrap()
            .into_iter()
            .map(|v| {
                (
                    layout.fields[v.field].name.clone(),
                    v.value,
                    hex(&v.octets),
                    if v.accepted { "accepted" } else { "refused" },
                )
            })
            .collect()
    }

    fn zero(layout: &Layout) -> Values {
        settle(layout, vec![None; layout.fields.len()])
    }

    #[test]
    fn the_dns_header_has_its_13_fields_in_96_bits() {
        let header = layout("dns");
        let fields: Vec<(&str, u32)> = header
            .fields
            .iter()
            .map(|f| (f.name.as_str(), f.bits))
            .collect();
        assert_eq!(
            fields,
            [
                ("id", 16),
                ("qr", 1),
                ("opcode", 4),
                ("aa", 1),
                ("tc", 1),
                ("rd", 1),
                ("ra", 1),
                ("z", 3),
                ("rcode", 4),
                ("qdcount", 16),
                ("ancount", 16),
                ("nscount", 16),
                ("arcount", 16),
            ]
        );
        assert_eq!(header.fields.iter().map(|f| f.bits).sum::<u32>(), 96);
        assert!(header.epoch.is_none() && header.length.is_empty());
        assert!(
            header
                .fields
                .iter()
                .all(|f| !f.signed && f.order == "big-endian" && f.presence.is_none())
        );
        let constrained: Vec<(&str, &[Constraint])> = header
            .fields
            .iter()
            .filter(|f| !f.constraints.is_empty())
            .map(|f| (f.name.as_str(), f.constraints.as_slice()))
            .collect();
        assert_eq!(constrained, [("z", &[Constraint::Equals(0)][..])]);
    }

    #[test]
    fn an_equals_constraint_yields_z_0_accepted_and_z_1_refused_on_the_zero_header() {
        let header = layout("dns");
        assert_eq!(
            listed(&header, &zero(&header)),
            [
                (
                    "z".to_owned(),
                    0,
                    "00 00 00 00 00 00 00 00 00 00 00 00".to_owned(),
                    "accepted"
                ),
                (
                    "z".to_owned(),
                    1,
                    "00 00 00 10 00 00 00 00 00 00 00 00".to_owned(),
                    "refused"
                ),
            ]
        );
    }

    #[test]
    fn an_equals_value_is_refused_on_each_side_its_width_holds() {
        let mut header = layout("dns");
        let z = index_of(&header, "z");
        let verdicts = |header: &Layout, value: u64| -> Vec<(u64, &'static str)> {
            let base = with(header, &zero(header), z, value);
            listed(header, &base)
                .into_iter()
                .map(|(_, value, _, verdict)| (value, verdict))
                .collect()
        };
        header.fields[z].constraints = vec![Constraint::Equals(3)];
        assert_eq!(
            verdicts(&header, 3),
            [(3, "accepted"), (4, "refused"), (2, "refused")]
        );
        header.fields[z].constraints = vec![Constraint::Equals(7)];
        assert_eq!(verdicts(&header, 7), [(7, "accepted"), (6, "refused")]);
    }

    #[test]
    fn the_websocket_layout_without_equals_constraints_yields_the_same_vectors_as_before() {
        let frame = layout("websocket");
        assert!(
            frame
                .fields
                .iter()
                .flat_map(|f| &f.constraints)
                .all(|c| !matches!(c, Constraint::Equals(_)))
        );
        let (base, _) = decode(&frame, &[0x81, 0x05]).unwrap();
        let vectors: Vec<String> = listed(&frame, &base)
            .into_iter()
            .map(|(name, value, octets, verdict)| format!("{name} {value} {verdict} {octets}"))
            .collect();
        assert_eq!(
            vectors,
            [
                "extended-payload-length-16 126 accepted 81 7e 00 7e",
                "extended-payload-length-16 65535 accepted 81 7e ff ff",
                "extended-payload-length-16 124 refused 81 7e 00 7c",
                "extended-payload-length-16 125 refused 81 7e 00 7d",
                "extended-payload-length-64 65536 accepted 81 7f 00 00 00 00 00 01 00 00",
                "extended-payload-length-64 9223372036854775807 accepted 81 7f 7f ff ff ff ff ff ff ff",
                "extended-payload-length-64 65534 refused 81 7f 00 00 00 00 00 00 ff fe",
                "extended-payload-length-64 65535 refused 81 7f 00 00 00 00 00 00 ff ff",
                "extended-payload-length-64 9223372036854775808 refused 81 7f 80 00 00 00 00 00 00 00",
                "mask 0 accepted 81 05",
                "payload-len 125 accepted 81 7d",
            ]
        );
    }
}
