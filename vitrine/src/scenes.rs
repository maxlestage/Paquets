//! Les scènes 3D, dans le navigateur : `paquets::three` (l'API de Three.js, en Rust)
//! dessine avec WebGL 2, `paquets::three::Physics` fait tomber les objets.
//!
//! Chaque scène tourne dans une `animation_loop` qui s'arrête quand l'îlot disparaît,
//! et ne dessine rien tant que son canvas est hors de l'écran.

use std::cell::Cell;
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::rc::Rc;

use paquets::prelude::State;
use paquets::three::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

use crate::{Reglages, Scene3d, TEINTES};

/// Une scène qui tourne : la lâcher l'arrête.
pub struct Running {
    _loop: Option<AnimationLoop>,
    _pointer: Option<Listener>,
}

/// Un écouteur d'événement sur la fenêtre, retiré quand on le lâche.
struct Listener {
    event: &'static str,
    closure: Closure<dyn FnMut(web_sys::PointerEvent)>,
}

impl Drop for Listener {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            let _ = window.remove_event_listener_with_callback(
                self.event,
                self.closure.as_ref().unchecked_ref(),
            );
        }
    }
}

/// Lance `scene` dans `canvas` ; `info` reçoit les images par seconde (ou le nombre de
/// paquets pour la pluie).
pub fn start(
    canvas: &paquets::active::Dom,
    scene: Scene3d,
    info: State<u32>,
    reglages: Reglages,
) -> Running {
    let mut renderer = match WebGLRenderer::new(canvas) {
        Ok(renderer) => renderer,
        Err(_) => {
            // Pas de WebGL 2 : la page garde son fond animé en CSS.
            let _ = canvas.class_list().add_1("no-webgl");
            return Running {
                _loop: None,
                _pointer: None,
            };
        }
    };
    renderer.set_pixel_ratio(device_pixel_ratio().min(2.0));
    let _ = canvas.class_list().add_1("live");
    let canvas = canvas.clone();
    match scene {
        Scene3d::Hero => hero(renderer, canvas, info),
        Scene3d::Atelier => atelier(renderer, canvas, info, reglages),
        Scene3d::Pluie => pluie(renderer, canvas, info, reglages),
    }
}

/// Moins de mouvement quand le visiteur le demande.
fn reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .is_some_and(|m| m.matches())
}

/// Si le canvas est au moins en partie à l'écran.
fn visible(canvas: &web_sys::Element) -> bool {
    let rect = canvas.get_bounding_client_rect();
    let height = web_sys::window()
        .and_then(|w| w.inner_height().ok())
        .and_then(|h| h.as_f64())
        .unwrap_or(1000.0);
    rect.bottom() > 0.0 && rect.top() < height && rect.width() > 0.0
}

/// Compte les images et donne le résultat à `info` chaque seconde.
struct Fps {
    frames: u32,
    since: f64,
}

impl Fps {
    fn new() -> Fps {
        Fps {
            frames: 0,
            since: 0.0,
        }
    }

    fn tick(&mut self, time: f64, info: State<u32>) {
        self.frames += 1;
        if self.since == 0.0 {
            self.since = time;
        } else if time - self.since >= 1000.0 {
            info.set((self.frames as f64 * 1000.0 / (time - self.since)).round() as u32);
            self.frames = 0;
            self.since = time;
        }
    }
}

/// Des nombres au hasard, assez pour placer et colorer des objets (xorshift).
struct Hasard(u64);

impl Hasard {
    fn new() -> Hasard {
        Hasard((js_sys_now() as u64) | 1)
    }

    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next()
    }

    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[((self.next() * items.len() as f32) as usize).min(items.len() - 1)]
    }
}

fn js_sys_now() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now() * 1000.0 + p.time_origin())
        .unwrap_or(12345.0)
}

const PALETTE: [u32; 5] = [0x7c5cff, 0xff6b6b, 0x2ee6a6, 0xffc857, 0x5cc8ff];

// ————————————————————————————————————————————————————————————————————————————
// Le héros
// ————————————————————————————————————————————————————————————————————————————

struct Satellite {
    id: ObjectId,
    rayon: f32,
    angle: f32,
    vitesse: f32,
    hauteur: f32,
    phase: f32,
    spin: Vector3,
}

fn hero(mut renderer: WebGLRenderer, canvas: web_sys::Element, info: State<u32>) -> Running {
    let calme = if reduced_motion() { 0.15 } else { 1.0 };
    let mut hasard = Hasard::new();
    let mut scene = Scene::new();

    let monde = scene.add(Group::new());

    // Le cœur : une gemme à facettes, métallique.
    let coeur = scene.add_to(
        monde,
        Mesh::new(
            SphereGeometry::new(1.25, 7, 5),
            MeshStandardMaterial::new()
                .color(0x7c5cff)
                .metalness(0.65)
                .roughness(0.22)
                .emissive(0x1a0f4d)
                .flat_shading(true),
        ),
    );
    // Sa coque en fil de fer.
    let coque = scene.add_to(
        monde,
        Mesh::new(
            SphereGeometry::new(1.95, 14, 10),
            MeshBasicMaterial::new()
                .color(0x9d86ff)
                .wireframe(true)
                .transparent(true)
                .opacity(0.28),
        ),
    );
    // Deux anneaux lumineux.
    let anneau_a = scene.add_to(
        monde,
        Mesh::new(
            TorusGeometry::new(2.7, 0.025, 8, 128),
            MeshBasicMaterial::new().color(0xff6b6b),
        ),
    );
    scene[anneau_a].rotation.x = FRAC_PI_2 * 0.82;
    let anneau_b = scene.add_to(
        monde,
        Mesh::new(
            TorusGeometry::new(3.3, 0.018, 8, 128),
            MeshBasicMaterial::new().color(0x2ee6a6),
        ),
    );
    scene[anneau_b].rotation.x = FRAC_PI_2 * 1.2;
    scene[anneau_b].rotation.y = 0.5;

    // Des paquets en orbite.
    let cube = BoxGeometry::new(0.22, 0.22, 0.22);
    let mut satellites = Vec::new();
    for _ in 0..46 {
        let couleur = hasard.pick(&PALETTE);
        let id = scene.add_to(
            monde,
            Mesh::new(
                cube.clone(),
                MeshStandardMaterial::new()
                    .color(couleur)
                    .metalness(0.3)
                    .roughness(0.35)
                    .emissive(Color::from(couleur).scale(0.25)),
            ),
        );
        let taille = hasard.range(0.5, 1.6);
        scene[id].scale.set(taille, taille, taille);
        satellites.push(Satellite {
            id,
            rayon: hasard.range(3.0, 6.4),
            angle: hasard.range(0.0, TAU),
            vitesse: hasard.range(0.08, 0.32) * if hasard.next() > 0.5 { 1.0 } else { -1.0 },
            hauteur: hasard.range(-1.8, 1.8),
            phase: hasard.range(0.0, TAU),
            spin: Vector3::new(
                hasard.range(-1.5, 1.5),
                hasard.range(-1.5, 1.5),
                hasard.range(-1.5, 1.5),
            ),
        });
    }

    // Un champ d'étoiles.
    let etoile = BoxGeometry::new(0.035, 0.035, 0.035);
    let ciel = scene.add(Group::new());
    for _ in 0..170 {
        let id = scene.add_to(
            ciel,
            Mesh::new(
                etoile.clone(),
                MeshBasicMaterial::new().color(hasard.pick(&[0xffffff, 0xc9bfff, 0x9fe8ff])),
            ),
        );
        let r = hasard.range(9.0, 22.0);
        let theta = hasard.range(0.0, TAU);
        let phi = hasard.range(0.2, PI - 0.2);
        scene[id].position.set(
            r * phi.sin() * theta.cos(),
            r * phi.cos(),
            r * phi.sin() * theta.sin() - 6.0,
        );
    }

    scene.add(AmbientLight::new(0xb4a8ff, 0.5));
    let soleil = scene.add(DirectionalLight::new(0xffffff, 2.6));
    scene[soleil].position.set(3.0, 4.0, 5.0);
    let corail = scene.add(PointLight::new(0xff6b6b, 40.0));
    scene[corail].position.set(-4.0, 1.5, 2.5);
    let menthe = scene.add(PointLight::new(0x2ee6a6, 30.0));
    scene[menthe].position.set(4.0, -2.0, 2.0);

    let mut camera = PerspectiveCamera::new(48.0, 0.0, 0.1, 100.0);
    camera.position.set(0.0, 0.0, 10.0);

    // Le pointeur, de -1 à 1 sur chaque axe.
    let pointeur = Rc::new(Cell::new((0.0f32, 0.0f32)));
    let listener = {
        let pointeur = pointeur.clone();
        let closure = Closure::<dyn FnMut(web_sys::PointerEvent)>::new(
            move |event: web_sys::PointerEvent| {
                if let Some(window) = web_sys::window() {
                    let w = window
                        .inner_width()
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0);
                    let h = window
                        .inner_height()
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0);
                    pointeur.set((
                        (event.client_x() as f64 / w * 2.0 - 1.0) as f32,
                        (event.client_y() as f64 / h * 2.0 - 1.0) as f32,
                    ));
                }
            },
        );
        web_sys::window().map(|window| {
            let _ = window
                .add_event_listener_with_callback("pointermove", closure.as_ref().unchecked_ref());
            Listener {
                event: "pointermove",
                closure,
            }
        })
    };

    let mut fps = Fps::new();
    let mut lisse = (0.0f32, 0.0f32);
    let mut defilement_lisse = 0.0f32;
    let mut decalage = 0.0f32;
    let mut avant = 0.0f64;
    let boucle = animation_loop(move |time| {
        let dt = if avant == 0.0 {
            0.016
        } else {
            ((time - avant) / 1000.0) as f32
        }
        .min(0.1);
        avant = time;
        if !visible(&canvas) {
            return;
        }
        fps.tick(time, info);
        let t = time as f32 / 1000.0 * calme;

        // Le pointeur et le défilement, adoucis.
        let (px, py) = pointeur.get();
        lisse.0 += (px - lisse.0) * (dt * 3.0).min(1.0);
        lisse.1 += (py - lisse.1) * (dt * 3.0).min(1.0);
        let defilement = web_sys::window()
            .and_then(|w| w.scroll_y().ok())
            .unwrap_or(0.0) as f32;
        defilement_lisse += (defilement - defilement_lisse) * (dt * 6.0).min(1.0);
        let s = (defilement_lisse / 900.0).min(1.5);

        camera.position.x = lisse.0 * 1.6;
        camera.position.y = -lisse.1 * 1.1 + s * 1.5;
        camera.position.z = 10.0 + s * 4.0;
        camera.look_at(Vector3::new(0.0, s * 0.8, 0.0));

        scene[monde].rotation.y = t * 0.12 + lisse.0 * 0.35 + s * 1.2;
        scene[monde].rotation.x = lisse.1 * 0.2;
        scene[monde].position.y = s * 1.2;
        // Sur un écran large, la scène se range à droite du titre.
        let aspect = renderer.aspect();
        let cible = if aspect > 1.15 {
            (aspect - 1.0).min(0.9) * 4.2
        } else {
            0.0
        };
        decalage += (cible - decalage) * (dt * 4.0).min(1.0);
        scene[monde].position.x = decalage;
        scene[coeur].rotation.y = t * 0.6;
        scene[coeur].rotation.x = t * 0.35;
        let pulse = 1.0 + (t * 2.0).sin() * 0.04;
        scene[coeur].scale.set(pulse, pulse, pulse);
        scene[coque].rotation.y = -t * 0.25;
        scene[coque].rotation.z = t * 0.1;
        scene[anneau_a].rotation.z = t * 0.4;
        scene[anneau_b].rotation.z = -t * 0.3;
        scene[ciel].rotation.y = t * 0.02;
        for sat in &satellites {
            let a = sat.angle + t * sat.vitesse;
            let o = &mut scene[sat.id];
            o.position.set(
                a.cos() * sat.rayon,
                sat.hauteur + (t * 0.8 + sat.phase).sin() * 0.35,
                a.sin() * sat.rayon,
            );
            o.rotation.x = t * sat.spin.x;
            o.rotation.y = t * sat.spin.y;
            o.rotation.z = t * sat.spin.z;
        }
        scene[corail].position.x = (t * 0.7).cos() * 4.5;
        scene[corail].position.z = (t * 0.7).sin() * 4.5;

        renderer.render(&scene, &camera);
    });
    Running {
        _loop: Some(boucle),
        _pointer: listener,
    }
}

// ————————————————————————————————————————————————————————————————————————————
// L'atelier
// ————————————————————————————————————————————————————————————————————————————

/// Une forme de l'atelier : son groupe et les meshes qui prennent la teinte.
struct Forme {
    groupe: ObjectId,
    teintes: Vec<ObjectId>,
}

fn standard(couleur: u32) -> MeshStandardMaterial {
    MeshStandardMaterial::new()
        .color(couleur)
        .metalness(0.85)
        .roughness(0.22)
}

fn atelier(
    mut renderer: WebGLRenderer,
    canvas: web_sys::Element,
    info: State<u32>,
    r: Reglages,
) -> Running {
    let mut scene = Scene::new();
    let piedestal = scene.add(Group::new());
    let socle = scene.add_to(
        piedestal,
        Mesh::new(
            CylinderGeometry::new(2.2, 2.45, 0.35, 72),
            MeshStandardMaterial::new()
                .color(0x15131f)
                .metalness(0.4)
                .roughness(0.5),
        ),
    );
    scene[socle].position.y = -1.75;
    let halo = scene.add_to(
        piedestal,
        Mesh::new(
            TorusGeometry::new(2.3, 0.03, 8, 128),
            MeshBasicMaterial::new().color(0x7c5cff),
        ),
    );
    scene[halo].position.y = -1.55;
    scene[halo].rotation.x = FRAC_PI_2;

    let base = TEINTES[0].1;
    let mut formes = Vec::new();

    // Le paquet : une boîte, deux rubans et un nœud.
    {
        let g = scene.add(Group::new());
        let boite = scene.add_to(
            g,
            Mesh::new(BoxGeometry::new(1.8, 1.8, 1.8), standard(base)),
        );
        let ruban = MeshStandardMaterial::new()
            .color(0xfff4e0)
            .metalness(0.2)
            .roughness(0.4);
        scene.add_to(
            g,
            Mesh::new(BoxGeometry::new(1.84, 1.84, 0.3), ruban.clone()),
        );
        scene.add_to(
            g,
            Mesh::new(BoxGeometry::new(0.3, 1.84, 1.84), ruban.clone()),
        );
        let noeud_a = scene.add_to(
            g,
            Mesh::new(TorusGeometry::new(0.32, 0.09, 12, 32), ruban.clone()),
        );
        scene[noeud_a].position.set(-0.3, 1.05, 0.0);
        scene[noeud_a].rotation.y = FRAC_PI_2;
        scene[noeud_a].rotation.z = 0.6;
        let noeud_b = scene.add_to(g, Mesh::new(TorusGeometry::new(0.32, 0.09, 12, 32), ruban));
        scene[noeud_b].position.set(0.3, 1.05, 0.0);
        scene[noeud_b].rotation.y = FRAC_PI_2;
        scene[noeud_b].rotation.z = -0.6;
        formes.push(Forme {
            groupe: g,
            teintes: vec![boite],
        });
    }
    // Le tore.
    {
        let g = scene.add(Group::new());
        let tore = scene.add_to(
            g,
            Mesh::new(TorusGeometry::new(1.1, 0.42, 32, 96), standard(base)),
        );
        scene[tore].rotation.x = 0.5;
        formes.push(Forme {
            groupe: g,
            teintes: vec![tore],
        });
    }
    // La gemme : deux cônes à facettes.
    {
        let g = scene.add(Group::new());
        let haut = scene.add_to(
            g,
            Mesh::new(
                ConeGeometry::new(1.2, 0.9, 8),
                standard(base).flat_shading(true),
            ),
        );
        scene[haut].position.y = 0.45;
        let bas = scene.add_to(
            g,
            Mesh::new(
                ConeGeometry::new(1.2, 1.9, 8),
                standard(base).flat_shading(true),
            ),
        );
        scene[bas].position.y = -0.95;
        scene[bas].rotation.x = PI;
        formes.push(Forme {
            groupe: g,
            teintes: vec![haut, bas],
        });
    }
    // La capsule : un cylindre et deux demi-sphères.
    {
        let g = scene.add(Group::new());
        let corps = scene.add_to(
            g,
            Mesh::new(CylinderGeometry::new(0.7, 0.7, 1.4, 48), standard(base)),
        );
        let haut = scene.add_to(
            g,
            Mesh::new(SphereGeometry::new(0.7, 48, 24), standard(base)),
        );
        scene[haut].position.y = 0.7;
        let bas = scene.add_to(
            g,
            Mesh::new(SphereGeometry::new(0.7, 48, 24), standard(base)),
        );
        scene[bas].position.y = -0.7;
        scene[g].rotation.z = 0.5;
        formes.push(Forme {
            groupe: g,
            teintes: vec![corps, haut, bas],
        });
    }

    scene.add(HemisphereLight::new(0xd9d0ff, 0x1a1530, 1.2));
    let cle = scene.add(DirectionalLight::new(0xffffff, 3.0));
    scene[cle].position.set(4.0, 6.0, 5.0);
    let contre = scene.add(PointLight::new(0xff6b6b, 35.0));
    scene[contre].position.set(-4.0, 1.0, -3.0);
    let rebond = scene.add(PointLight::new(0x5cc8ff, 25.0));
    scene[rebond].position.set(3.5, -1.0, 3.0);

    let mut camera = PerspectiveCamera::new(42.0, 0.0, 0.1, 100.0);
    camera.position.set(4.2, 2.4, 6.2);
    camera.look_at(Vector3::ZERO);
    let mut controls = OrbitControls::new(&canvas);
    controls.enable_damping = true;
    controls.enable_pan = false;
    controls.min_distance = 4.5;
    controls.max_distance = 13.0;
    controls.auto_rotate_speed = 3.0;

    // Ce qui est montré, pour ne changer les matériaux que quand il le faut.
    let mut montre: Option<(usize, usize, bool, bool)> = None;
    let mut fps = Fps::new();
    let mut apparition = 1.0f32;
    let boucle = animation_loop(move |time| {
        if !visible(&canvas) {
            return;
        }
        fps.tick(time, info);
        let t = time as f32 / 1000.0;
        let voulu = (
            r.forme.get(),
            r.teinte.get(),
            r.metal.get(),
            r.filaire.get(),
        );
        if montre != Some(voulu) {
            let (forme, teinte, metal, filaire) = voulu;
            if montre.map(|m| m.0) != Some(forme) {
                apparition = 0.0;
            }
            let couleur = Color::from(TEINTES[teinte.min(TEINTES.len() - 1)].1);
            for (i, f) in formes.iter().enumerate() {
                scene[f.groupe].visible = i == forme;
                for id in &f.teintes {
                    if let Some(mesh) = scene[*id].mesh_mut()
                        && let Material::Standard(m) = &mut mesh.material
                    {
                        m.surface.color = couleur;
                        m.surface.wireframe = filaire;
                        m.metalness = if metal { 0.85 } else { 0.05 };
                        m.roughness = if metal { 0.2 } else { 0.55 };
                        m.emissive = couleur.scale(if filaire { 0.6 } else { 0.08 });
                    }
                }
            }
            if let Some(mesh) = scene[halo].mesh_mut() {
                mesh.material.surface_mut().color = couleur;
            }
            montre = Some(voulu);
        }
        // Une apparition élastique quand la forme change.
        apparition = (apparition + 0.035).min(1.0);
        let e = elastique(apparition);
        let f = &formes[voulu.0.min(formes.len() - 1)];
        scene[f.groupe].scale.set(e, e, e);
        scene[f.groupe].position.y = (t * 1.4).sin() * 0.12 + 0.1;
        scene[f.groupe].rotation.y = t * 0.3;
        scene[halo].scale.set(
            1.0 + (t * 2.0).sin() * 0.02,
            1.0 + (t * 2.0).sin() * 0.02,
            1.0,
        );
        controls.auto_rotate = r.auto.get();
        controls.update(&mut camera);
        renderer.render(&scene, &camera);
    });
    Running {
        _loop: Some(boucle),
        _pointer: None,
    }
}

/// De 0 à 1, en dépassant un peu (easeOutElastic adouci).
fn elastique(x: f32) -> f32 {
    if x >= 1.0 {
        return 1.0;
    }
    let c4 = TAU / 3.0;
    (2.0f32.powf(-10.0 * x) * ((x * 10.0 - 0.75) * c4).sin() * 0.6 + 1.0).max(0.0)
}

// ————————————————————————————————————————————————————————————————————————————
// La pluie de paquets
// ————————————————————————————————————————————————————————————————————————————

const MAX_PAQUETS: usize = 140;

fn pluie(
    mut renderer: WebGLRenderer,
    canvas: web_sys::Element,
    info: State<u32>,
    r: Reglages,
) -> Running {
    let mut hasard = Hasard::new();
    let mut scene = Scene::new();
    let sol = scene.add(Mesh::new(
        PlaneGeometry::new(14.0, 14.0),
        MeshStandardMaterial::new()
            .color(0x1b1830)
            .roughness(0.85)
            .metalness(0.1),
    ));
    scene[sol].rotation.x = -FRAC_PI_2;

    scene.add(HemisphereLight::new(0xe6e0ff, 0x221d3a, 1.4));
    let soleil = scene.add(DirectionalLight::new(0xffffff, 2.8));
    scene[soleil].position.set(5.0, 10.0, 6.0);
    let corail = scene.add(PointLight::new(0xff6b6b, 40.0));
    scene[corail].position.set(-5.0, 4.0, 3.0);

    let mut camera = PerspectiveCamera::new(45.0, 0.0, 0.1, 100.0);
    camera.position.set(9.0, 7.5, 11.0);
    camera.look_at(Vector3::new(0.0, 1.0, 0.0));
    let mut controls = OrbitControls::new(&canvas);
    controls.target = Vector3::new(0.0, 1.0, 0.0);
    controls.enable_damping = true;
    controls.enable_pan = false;
    controls.min_distance = 8.0;
    controls.max_distance = 26.0;
    controls.max_polar_angle = FRAC_PI_2 * 0.92;
    controls.auto_rotate = !reduced_motion();
    controls.auto_rotate_speed = 0.8;

    let tailles = [
        BoxGeometry::new(0.6, 0.6, 0.6),
        BoxGeometry::new(0.9, 0.6, 0.6),
        BoxGeometry::new(0.8, 0.8, 0.8),
        BoxGeometry::new(1.0, 0.5, 0.7),
    ];
    let mut physics = Physics::new();
    let mut paquets: Vec<ObjectId> = Vec::new();
    // Quelques paquets pour commencer, puis ceux que l'on demande.
    let mut a_lacher = 16u32;
    let mut lache_vu = r.lacher.get();
    let mut secousse_vue = r.secousse.get();
    let mut minuterie = 0.0f32;
    let mut avant = 0.0f64;
    let boucle = animation_loop(move |time| {
        let dt = if avant == 0.0 {
            0.016
        } else {
            ((time - avant) / 1000.0) as f32
        }
        .min(0.05);
        avant = time;
        if !visible(&canvas) {
            return;
        }
        let demande = r.lacher.get();
        if demande != lache_vu {
            a_lacher += demande.saturating_sub(lache_vu);
            lache_vu = demande;
        }
        // Un paquet toutes les 60 ms, en pluie plutôt qu'en bloc.
        minuterie -= dt;
        if a_lacher > 0 && minuterie <= 0.0 {
            minuterie = 0.06;
            a_lacher -= 1;
            let couleur = hasard.pick(&PALETTE);
            let id = scene.add(Mesh::new(
                tailles[(hasard.next() * tailles.len() as f32) as usize % tailles.len()].clone(),
                MeshStandardMaterial::new()
                    .color(couleur)
                    .metalness(0.25)
                    .roughness(0.4)
                    .emissive(Color::from(couleur).scale(0.12)),
            ));
            let o = &mut scene[id];
            o.position.set(
                hasard.range(-2.5, 2.5),
                hasard.range(7.0, 10.0),
                hasard.range(-2.5, 2.5),
            );
            o.rotation.set(
                hasard.range(0.0, TAU),
                hasard.range(0.0, TAU),
                hasard.range(0.0, TAU),
            );
            paquets.push(id);
            if paquets.len() > MAX_PAQUETS {
                let vieux = paquets.remove(0);
                scene.remove(vieux);
            }
        }
        physics.step(&mut scene, dt);
        let secousse = r.secousse.get();
        if secousse != secousse_vue {
            secousse_vue = secousse;
            for id in &paquets {
                if let Some(body) = physics.body_mut(*id) {
                    body.wake_up();
                    let m = body.mass();
                    body.apply_impulse(
                        Vector3::new(
                            hasard.range(-2.0, 2.0),
                            hasard.range(5.0, 8.0),
                            hasard.range(-2.0, 2.0),
                        ) * m,
                    );
                }
            }
        }
        // Ceux qui tombent du plateau disparaissent.
        paquets.retain(|id| {
            let tombe = scene.get(*id).is_some_and(|o| o.position.y < -12.0);
            if tombe {
                scene.remove(*id);
            }
            !tombe
        });
        if info.get() != paquets.len() as u32 {
            info.set(paquets.len() as u32);
        }
        controls.update(&mut camera);
        renderer.render(&scene, &camera);
    });
    Running {
        _loop: Some(boucle),
        _pointer: None,
    }
}
