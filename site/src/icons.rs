//! Line icons (24×24, stroked) and the Paquets logo, as active views: the same
//! markup on the server and in islands.

use paquets::prelude::*;

/// The stroke paths of an icon.
pub fn paths(name: &str) -> &'static [&'static str] {
    match name {
        "bolt" => &["M13 2 4 14h7l-1 8 9-12h-7z"],
        "shield" => &[
            "M12 3 4 6v6c0 5 3.5 8 8 9 4.5-1 8-4 8-9V6z",
            "m9 12 2 2 4-4",
        ],
        "db" => &[
            "M4 6c0-1.7 3.6-3 8-3s8 1.3 8 3-3.6 3-8 3-8-1.3-8-3z",
            "M4 6v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6",
            "M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3",
        ],
        "spark" => &[
            "M12 3c.8 4.5 2.5 6.2 7 7-4.5.8-6.2 2.5-7 7-.8-4.5-2.5-6.2-7-7 4.5-.8 6.2-2.5 7-7z",
            "M19 15v5",
            "M16.5 17.5h5",
        ],
        "box" => &[
            "M12 2.5 3.5 7v10l8.5 4.5 8.5-4.5V7z",
            "m3.5 7 8.5 4.5L20.5 7",
            "M12 11.5v10",
            "m7.8 4.7 8.5 4.6",
        ],
        "page" => &["M14 3H6v18h12V7z", "M14 3v4h4", "M9 12h6", "M9 16h6"],
        "phone" => &[
            "M8 2h8a2 2 0 0 1 2 2v16a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z",
            "M11 18h2",
        ],
        "flask" => &["M9 3h6", "M10 3v6L4 20h16L14 9V3", "M7 15h10"],
        "crate" => &["M3 4h18v4H3z", "M5 8v12h14V8", "M10 12h4"],
        "rocket" => &[
            "M5 16c-1.5 1.2-2 5-2 5s3.8-.5 5-2c.7-.8.7-2.1-.1-2.9A2.2 2.2 0 0 0 5 16z",
            "m12 15-3-3a22 22 0 0 1 2-4A13 13 0 0 1 22 2c0 2.7-.8 7.5-6 11a22 22 0 0 1-4 2z",
            "M9 12H4s.6-3 2-4c1.6-1.1 5 0 5 0",
            "M12 15v5s3-.6 4-2c1.1-1.6 0-5 0-5",
        ],
        "layers" => &["m12 3 9 5-9 5-9-5z", "m3 13 9 5 9-5"],
        "cloud" => &["M7 19h10a4.5 4.5 0 0 0 .6-9A6.5 6.5 0 0 0 5.2 10.4 4.3 4.3 0 0 0 7 19z"],
        "book" => &[
            "M4 5.5A2.5 2.5 0 0 1 6.5 3H20v15H6.5A2.5 2.5 0 0 0 4 20.5z",
            "M4 20.5A2.5 2.5 0 0 0 6.5 23H20v-5",
            "M8 7h8",
        ],
        "git" => &[
            "M6 3v12",
            "M18 9a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
            "M6 21a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
            "M18 9a9 9 0 0 1-9 9",
        ],
        "terminal" => &["m4 17 6-6-6-6", "M12 19h8"],
        "external" => &["M14 4h6v6", "M20 4 10 14", "M18 14v6H4V6h6"],
        "arrow" => &["M5 12h14", "m13 6 6 6-6 6"],
        "up" => &["M12 19V5", "m6 11 6-6 6 6"],
        "search" => &["M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14z", "m20 20-4.2-4.2"],
        "edit" => &["M4 20h4L19 9l-4-4L4 16z", "m13.5 6.5 4 4"],
        "menu" => &["M4 7h16", "M4 12h16", "M4 17h10"],
        "copy" => &["M9 9h11v11H9z", "M5 15H4V4h11v1"],
        "check" => &["m5 12 5 5L20 7"],
        "replay" => &["M3 12a9 9 0 1 0 3-6.7", "M3 4v5h5"],
        "close" => &["M6 6l12 12", "M18 6 6 18"],
        "globe" => &[
            "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z",
            "M3 12h18",
            "M12 3c2.5 2.5 3.8 5.5 3.8 9s-1.3 6.5-3.8 9c-2.5-2.5-3.8-5.5-3.8-9S9.5 5.5 12 3z",
        ],
        _ => &[],
    }
}

/// An icon: `<svg class="icon">`.
pub fn icon(name: &str) -> Element {
    svg()
        .class("icon")
        .attr("viewBox", "0 0 24 24")
        .attr("fill", "none")
        .attr("stroke", "currentColor")
        .attr("stroke-width", "1.6")
        .attr("stroke-linecap", "round")
        .attr("stroke-linejoin", "round")
        .attr("aria-hidden", "true")
        .children(paths(name).iter().map(|d| path().attr("d", *d)))
}

fn gradient(
    id: &'static str,
    (x1, y1, x2, y2): (&'static str, &'static str, &'static str, &'static str),
    from: &'static str,
    to: &'static str,
) -> Element {
    linear_gradient()
        .id(id)
        .attr("x1", x1)
        .attr("y1", y1)
        .attr("x2", x2)
        .attr("y2", y2)
        .attr("gradientUnits", "userSpaceOnUse")
        .child(stop().attr("offset", "0").attr("stop-color", from))
        .child(stop().attr("offset", "1").attr("stop-color", to))
}

/// The logo: the Paquets box (the same drawing as `public/assets/logo.svg`).
pub fn logo() -> Element {
    let face = |d: &'static str, paint: &'static str| {
        path()
            .attr("d", d)
            .attr("fill", paint)
            .attr("stroke", paint)
    };
    svg()
        .class("logo-mark")
        .attr("viewBox", "0 0 64 64")
        .attr("aria-hidden", "true")
        .child(
            defs()
                .child(gradient(
                    "lg-top",
                    ("8", "18", "56", "18"),
                    "#8f74ff",
                    "#22d3ee",
                ))
                .child(gradient(
                    "lg-left",
                    ("8", "18", "32", "60"),
                    "#6d4aff",
                    "#3a1fb8",
                ))
                .child(gradient(
                    "lg-right",
                    ("32", "32", "56", "56"),
                    "#ff4fa3",
                    "#b72a7c",
                )),
        )
        .child(
            g().attr("stroke-linejoin", "round")
                .attr("stroke-width", "2")
                .child(face("M32 4 56 18 32 32 8 18Z", "url(#lg-top)"))
                .child(face("M8 18 32 32 32 60 8 46Z", "url(#lg-left)"))
                .child(face("M32 32 56 18 56 46 32 60Z", "url(#lg-right)")),
        )
        .child(
            path()
                .attr("d", "M17.4 12.5 22.6 9.5 46.6 23.5 41.4 26.5Z")
                .attr("fill", "#fff")
                .attr("fill-opacity", ".9"),
        )
        .child(
            path()
                .attr("d", "M41.4 26.5 46.6 23.5 46.6 51.5 41.4 54.5Z")
                .attr("fill", "#fff")
                .attr("fill-opacity", ".55"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_render() {
        let html = Node::from(icon("arrow")).render();
        assert!(html.starts_with("<svg class=\"icon\" viewBox=\"0 0 24 24\""));
        assert_eq!(html.matches("<path").count(), 2);
        assert!(paths("nope").is_empty());
        let logo = Node::from(logo()).render();
        assert!(logo.contains("linearGradient") && logo.contains("url(#lg-top)"));
    }
}
