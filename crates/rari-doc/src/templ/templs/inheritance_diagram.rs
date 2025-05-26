use std::fmt::{Error, Write};
use std::iter::once;

use rari_templ_func::rari_f;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::helpers::api_inheritance::inheritance;

#[rari_f(register = "crate::Templ")]
pub fn inheritance_diagram(interface: Option<String>) -> Result<String, DocError> {
    let main_if = interface
        .as_deref()
        .or_else(|| {
            env.slug
                .strip_prefix("Web/API/")
                .map(|s| &s[..s.find('/').unwrap_or(s.len())])
        })
        .ok_or_else(|| DocError::InvalidSlugForX(env.slug.to_string()))?;
    let inheritance_chain = inheritance(main_if);

    if inheritance_chain.is_empty() {
        return Ok(Default::default());
    }

    let mut out = String::new();
    let iter = inheritance_chain
        .iter()
        .rev()
        .chain(once(&main_if))
        .enumerate();
    let iter_len = inheritance_chain.len() + 1;

    let mut reverse = false;
    let mut height = 40;
    let mut x_pos = 0;
    let mut y_pos = 0;
    let mut left = 0;
    let mut right = 0;
    for (i, interface) in iter {
        let fill = if *interface == main_if {
            "#F4F7F8"
        } else {
            "#fff"
        };
        let rect_width = calculate_rect_width(interface);

        // Minimum space required to continue the current row
        let req_space = match i {
            0 => rect_width,
            _ if i == iter_len => rect_width + 47,
            _ => rect_width + 30,
        };

        // If the rect from the next iteration won't fit in the row, we need to be
        // sure that at least the connectingLineWithTriangle will fit
        let req_bounds = include_in_range(move_by(x_pos, req_space, reverse), left, right);

        // Will the current drawing items fit inside the viewbox? Subtract the
        // stroke width from the viewbox width to prevent stroke cut off.
        let can_continue_row = req_bounds.1 - req_bounds.0 <= 650 - 2;

        if i > 0 {
            if can_continue_row {
                line_with_triangle(&mut out, x_pos, y_pos + 14, reverse)?;
                x_pos = move_by(x_pos, 30, reverse);
            } else {
                connecting_line_with_triangle(&mut out, x_pos, y_pos + 14, reverse)?;
                (left, right) = include_in_range(move_by(x_pos, 17, reverse), left, right);
                y_pos += 46;
                height += 46;
                reverse = !reverse;
            }
        }
        rect_with_text(&mut out, x_pos, y_pos, fill, interface, reverse, env.locale)?;
        x_pos = move_by(x_pos, rect_width, reverse);
        (left, right) = include_in_range(x_pos, left, right);
    }

    Ok(format!(
        r#"<svg viewbox="{x} -1 650 {height}" preserveAspectRatio="xMinYMin meet">{diagram}</svg>"#,
        x = left - 1,
        height = height + 2,
        diagram = out,
    ))
}

fn rect_with_text(
    out: &mut String,
    x: i32,
    y: i32,
    fill: &str,
    interface_name: &str,
    reverse: bool,
    locale: Locale,
) -> Result<(), Error> {
    let rect_width = calculate_rect_width(interface_name);
    let x = if reverse { x - rect_width } else { x };
    write!(
        out,
        r##"<a style="text-decoration: none;" href="/{locale}/docs/Web/API/{interface_name}">
        <rect x="{x}" y="{y}" width="{rect_width}" height="25" fill="{fill}" stroke="#D4DDE4" stroke-width="2px" />
        <text x="{text_x}" y="{text_y}" font-size="10px" fill="#4D4E53" text-anchor="middle">
          {interface_name}
        </text>
      </a>"##,
        locale = locale.as_url_str(),
        interface_name = interface_name,
        x = x,
        y = y,
        rect_width = rect_width,
        fill = fill,
        text_x = f64::from(x) + f64::from(rect_width) / 2.0,
        text_y = y + 16
    )
}

fn line_with_triangle(out: &mut String, x: i32, y: i32, reverse: bool) -> Result<(), Error> {
    let length = if reverse { -30 } else { 30 };
    write!(
        out,
        r##"<line x1="{x}" y1="{y}" x2="{x2}" y2="{y2}" stroke="#D4DDE4"/>"##,
        x = x,
        y = y,
        x2 = x + length,
        y2 = y,
    )?;
    triangle(out, x, y, reverse)
}

fn connecting_line_with_triangle(
    out: &mut String,
    x: i32,
    y: i32,
    reverse: bool,
) -> Result<(), Error> {
    let width = if reverse { -17 } else { 17 };
    write!(
        out,
        r##"<polyline points="{x},{y} {x2},{y} {x2},{y2} {x},{y2}" stroke="#D4DDE4" fill="none"/>"##,
        x = x,
        y = y,
        x2 = x + width,
        y2 = y + 45,
    )?;
    triangle(out, x, y, reverse)
}

fn triangle(out: &mut String, x: i32, y: i32, reverse: bool) -> Result<(), Error> {
    let width = if reverse { -10 } else { 10 };
    write!(
        out,
        r##"<polyline points="{x},{y} {x2},{y2} {x2},{y3} {x},{y}" stroke="#D4DDE4" fill="#fff"/>"##,
        x = x,
        y = y,
        x2 = x + width,
        y2 = y - 5,
        y3 = y + 5
    )
}

fn calculate_rect_width(text: &str) -> i32 {
    let width = text.len() as i32 * 8;
    if width < 75 {
        75
    } else {
        width
    }
}

fn move_by(number: i32, delta: i32, reverse: bool) -> i32 {
    if reverse {
        number - delta
    } else {
        number + delta
    }
}

fn include_in_range(number: i32, start: i32, end: i32) -> (i32, i32) {
    (start.min(number), end.max(number))
}
