//! CSS 3D cubes: a package cube, the Paquets box (with a lid that opens),
//! the hero scene where the four packages fly into the box, and floating
//! background cubes. Everything moves in CSS; Rust only builds the faces.

use yew::prelude::*;

use super::ui::Logo;
use crate::data::PACKAGES;

const FACES: [&str; 6] = ["front", "back", "right", "left", "top", "bottom"];

#[derive(Properties, PartialEq)]
pub struct CubeProps {
    /// Edge length, in pixels; `0` leaves it to the CSS (`--s`).
    #[prop_or(56)]
    pub size: u32,
    /// Accent color; the Paquets gradient when absent.
    #[prop_or_default]
    pub color: Option<AttrValue>,
    /// A letter drawn on every face.
    #[prop_or_default]
    pub glyph: Option<AttrValue>,
    /// Turns on itself.
    #[prop_or_default]
    pub spin: bool,
    /// Transparent faces with glowing edges.
    #[prop_or_default]
    pub wire: bool,
    #[prop_or_default]
    pub class: Classes,
}

/// A cube with six faces.
#[component]
pub fn Cube(props: &CubeProps) -> Html {
    let mut style = String::new();
    if props.size > 0 {
        style.push_str(&format!("--s:{}px;", props.size));
    }
    if let Some(color) = &props.color {
        style.push_str(&format!("--c:{color};"));
    }
    html! {
        <div class={classes!(
            "cube",
            props.color.is_none().then_some("cube--brand"),
            props.wire.then_some("cube--wire"),
            props.class.clone()
        )} {style} aria-hidden="true">
            <div class={classes!("cube__body", props.spin.then_some("cube__body--spin"))}>
                { for FACES.iter().map(|face| html! {
                    <span class={classes!("cube__face", format!("cube__face--{face}"))}>
                        if let Some(glyph) = &props.glyph {
                            <b>{ glyph.clone() }</b>
                        }
                    </span>
                }) }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct BoxProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub children: Html,
}

/// The Paquets box: five glass faces and a lid hinged on its back edge.
/// `children` are placed inside (they share its 3D space).
#[component]
pub fn PaquetsBox(props: &BoxProps) -> Html {
    html! {
        <div class={classes!("pbox", props.class.clone())} aria-hidden="true">
            <span class="pbox__face pbox__face--front"><span class="pbox__tape"></span><span class="pbox__mark"><Logo /></span></span>
            <span class="pbox__face pbox__face--back"></span>
            <span class="pbox__face pbox__face--right"><span class="pbox__tape"></span></span>
            <span class="pbox__face pbox__face--left"></span>
            <span class="pbox__face pbox__face--bottom"></span>
            <span class="pbox__lid"><span class="pbox__tape pbox__tape--lid"></span></span>
            <span class="pbox__glow"></span>
            { props.children.clone() }
        </div>
    }
}

/// The hero scene: the four packages orbit, dive into the box one after
/// the other, the lid closes and the box lights up, then it starts again.
#[component]
pub fn Assembly() -> Html {
    html! {
        <div class="scene" aria-hidden="true">
            <div class="scene__halo"></div>
            <div class="scene__rig">
                <PaquetsBox>
                    { for PACKAGES.iter().enumerate().map(|(i, p)| html! {
                        <div class={classes!("pkg", format!("pkg--{i}"))} style={format!("--i:{i}")}>
                            <Cube size={0} color={Some(AttrValue::from(p.color))} glyph={Some(AttrValue::from(p.glyph))} />
                        </div>
                    }) }
                </PaquetsBox>
            </div>
            <div class="scene__floor"></div>
            <ul class="scene__labels">
                { for PACKAGES.iter().enumerate().map(|(i, p)| html! {
                    <li style={format!("--i:{i}; --c:{}", p.color)}><i></i>{ p.name }</li>
                }) }
            </ul>
        </div>
    }
}

/// Small cubes floating in the background, with parallax.
#[component]
pub fn FloatingCubes() -> Html {
    // (left %, top %, size px, parallax speed, wireframe)
    const CUBES: [(f64, f64, u32, f64, bool); 7] = [
        (6.0, 22.0, 34, 0.18, true),
        (88.0, 16.0, 26, -0.12, false),
        (14.0, 78.0, 22, 0.08, false),
        (78.0, 82.0, 40, 0.22, true),
        (46.0, 8.0, 18, -0.2, true),
        (94.0, 52.0, 16, 0.3, false),
        (36.0, 92.0, 28, -0.1, true),
    ];
    html! {
        <div class="floaters" aria-hidden="true">
            { for CUBES.iter().enumerate().map(|(i, (x, y, size, speed, wire))| html! {
                <div class="floater" data-speed={speed.to_string()}
                     style={format!("left:{x}%;top:{y}%;--i:{i}")}>
                    <Cube size={*size} wire={*wire} spin=true />
                </div>
            }) }
        </div>
    }
}
