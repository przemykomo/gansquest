use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gansui::App;
use gansui::draw::draw_tiled;
use gansui::element::Length;
use gansui::element::{Axis, Element};
use gansui::indextree::NodeId;
use gansui::is_in;
use gansui::sdl3::event::Event;
use gansui::sdl3::mouse::MouseButton::Left;
use gansui::sdl3::mouse::SystemCursor;
use gansui::sdl3::pixels::{Color, FColor};
use gansui::sdl3::rect::Rect;
use gansui::sdl3::render::FRect;
use gansui::sdl3::render::{FPoint, Vertex};
use gansui::sdl3::sys::events::SDL_EventType;
use gansui::sdl3::{self, timer};

use crate::{
    BLUE_GRAY, DARK_GRAY, DARK_PINK, GRAY_PINK, GREEN, LIGHT_BLUE, LIGHT_GRAY, WHITE_OVERLAY,
    YELLOW,
};
const NODE_LENGTH: f32 = 20.0;

pub struct QuestNode {
    x: f32,
    y: f32,
    prerequisites: Vec<usize>,
    state: QuestState,
    pub title_split: usize,
    // pub title: &'a str,
    // pub desc: &'a str,
    pub content: String,
}

impl QuestNode {
    pub fn new(
        x: f32,
        y: f32,
        prerequisites: Vec<usize>,
        state: QuestState,
        content: String,
    ) -> QuestNode {
        QuestNode {
            x,
            y,
            prerequisites,
            state,
            title_split: content.find('\n').unwrap_or(content.len()),
            content,
        }
    }

    pub fn set_content(&mut self, content: String) {
        self.title_split = content.find('\n').unwrap_or(content.len());
        self.content = content;
    }
}

pub enum QuestState {
    Unavailable,
    Available,
    Done,
}

pub fn load_graph<'a>() -> Rc<RefCell<HashMap<usize, QuestNode>>> {
    let mut nodes: HashMap<usize, QuestNode> = HashMap::new();
    nodes.insert(
        0,
        QuestNode::new(
            0.0,
            -50.0,
            vec![],
            QuestState::Done,
            "Node 0\ntest description!".to_owned(),
        ),
    );

    // nodes.insert(
    //     1,
    //     QuestNode {
    //         x: 0.0,
    //         y: 50.0,
    //         prerequisites: vec![0],
    //         state: QuestState::Done,
    //         // title: "Node 1 title",
    //         desc: "Node 1\ntest description".to_owned(),
    //     },
    // );
    //
    // nodes.insert(
    //     2,
    //     QuestNode {
    //         x: 50.0,
    //         y: 0.0,
    //         prerequisites: vec![0],
    //         state: QuestState::Available,
    //         // title: "Node 2 title",
    //         desc: "Node 2\ntest description".to_owned(),
    //     },
    // );
    //
    // nodes.insert(
    //     3,
    //     QuestNode {
    //         x: 100.0,
    //         y: 0.0,
    //         prerequisites: vec![1, 2],
    //         state: QuestState::Unavailable,
    //         // title: "Node 3 title",
    //         desc: "Node 3\ntest description".to_owned(),
    //     },
    // );
    //
    // nodes.insert(
    //     4,
    //     QuestNode {
    //         x: 150.0,
    //         y: 0.0,
    //         prerequisites: vec![3],
    //         state: QuestState::Unavailable,
    //         // title: "Node 4 title",
    //         desc: "Node 4\ntest description".to_owned(),
    //     },
    // );
    //
    // nodes.insert(
    //     5,
    //     QuestNode {
    //         x: 150.0,
    //         y: 150.0,
    //         prerequisites: vec![],
    //         state: QuestState::Available,
    //         // title: "Node 5 title",
    //         desc: "Node 5\ntest description".to_owned(),
    //     },
    // );

    Rc::new(RefCell::new(nodes))
}

pub fn generate_graph<'a>(
    atlas_txt: Rc<RefCell<sdl3::render::Texture>>,
    mut update_overlay: impl FnMut(&mut App) + 'a,
    nodes: Rc<RefCell<HashMap<usize, QuestNode>>>,
    selected: Rc<RefCell<Option<usize>>>,
) -> Element<'a> {
    let zoom: Rc<RefCell<f32>> = Rc::new(RefCell::new(1.0));
    let pos: Rc<RefCell<FPoint>> = Rc::new(RefCell::new(FPoint::new(0.0, 0.0)));
    let drag_started_on_graph = Rc::new(RefCell::new(false));

    let graph = Element::new()
        .with_width(Length::Grow {
            min: 0.0,
            max: f32::MAX,
        })
        .with_height(Length::Grow {
            min: 0.0,
            max: f32::MAX,
        })
        .with_background_color(BLUE_GRAY)
        .with_text_color(Color::WHITE)
        .with_layout_axis(Axis::Horizontal)
        .with_spacing(10.0)
        .on_mouse_down({
            let selected = selected.clone();
            let drag_started_on_graph = drag_started_on_graph.clone();
            move |app, element, button, clicks, mouse_x, mouse_y, _event| {
                if button == Left {
                    if is_in(mouse_x, mouse_y, &app.tree[element].get().aabb) {
                        *drag_started_on_graph.borrow_mut() = true;
                    }
                    let selected = selected.borrow_mut();
                    if let Some(_) = &*selected
                        && clicks == 2
                    {
                        drop(selected);
                        update_overlay(app);
                        app.set_event_handled();
                    }
                }
            }
        })
        .on_mouse_up({
            let drag_started_on_graph = drag_started_on_graph.clone();
            move |_app, _element, button, _clicks, _x, _y, _event| {
                if button == Left {
                    *drag_started_on_graph.borrow_mut() = false;
                }
            }
        })
        .on_mouse_motion({
            let pos = pos.clone();
            let zoom = zoom.clone();
            let nodes = nodes.clone();
            let selected = selected.clone();
            let drag_started_on_graph = drag_started_on_graph.clone();
            move |app,
                  element,
                  mouse_x,
                  mouse_y,
                  diff_x,
                  diff_y,
                  mousestate,
                  _event,
                  _update_selection| {
                let element = app.tree[element].get();
                if is_in(mouse_x, mouse_y, &element.aabb) {
                    let mut nodes = nodes.borrow_mut();
                    let zoom = zoom.borrow();
                    let mut pos = pos.borrow_mut();
                    let x = element.aabb.x + element.aabb.w / 2.0 + pos.x;
                    let y = element.aabb.y + element.aabb.h / 2.0 + pos.y;
                    let length = 20.0 * *zoom;
                    let mut selected = selected.borrow_mut();
                    if !mousestate.is_mouse_button_pressed(Left) {
                        *selected = None;
                        for (id, node) in nodes.iter() {
                            let node_aabb =
                                FRect::new(x + node.x * *zoom, y + node.y * *zoom, length, length);

                            if is_in(mouse_x, mouse_y, &node_aabb) {
                                app.cursor = SystemCursor::Hand;
                                *selected = Some(*id);
                                break;
                            }
                        }
                    }

                    // if mousestate.is_mouse_button_pressed(Left) {
                    if *drag_started_on_graph.borrow() {
                        if let Some(selected) = &*selected {
                            let node = nodes.get_mut(selected).unwrap();
                            let mouse_x = mouse_x - element.aabb.x - element.aabb.w / 2.0 - pos.x;
                            let mouse_y = mouse_y - element.aabb.y - element.aabb.h / 2.0 - pos.y;
                            node.x = mouse_x / *zoom - NODE_LENGTH / 2.0;
                            node.y = mouse_y / *zoom - NODE_LENGTH / 2.0;
                            if !app
                                .event_pump
                                .keyboard_state()
                                .is_scancode_pressed(sdl3::keyboard::Scancode::LShift)
                            {
                                const SNAP_LENGTH: f32 = NODE_LENGTH * 0.6;
                                node.x = (node.x / SNAP_LENGTH).round() * SNAP_LENGTH;
                                node.y = (node.y / SNAP_LENGTH).round() * SNAP_LENGTH;
                            }
                        } else {
                            pos.x += diff_x;
                            pos.y += diff_y;
                        }
                    }
                }
            }
        })
        .on_event({
            let pos = pos.clone();
            let zoom = zoom.clone();
            move |app: &mut App, element: NodeId, event: &Event| {
                let element = app.tree[element].get_mut();
                match event {
                    Event::MouseWheel {
                        timestamp: _,
                        window_id: _,
                        which: _,
                        x: _,
                        y: scrolled,
                        direction: _,
                        mouse_x,
                        mouse_y,
                    } => {
                        if is_in(*mouse_x, *mouse_y, &element.aabb) {
                            let factor = 1.2f32.powf(1.0 / scrolled);
                            zoom_graph(
                                pos.clone(),
                                zoom.clone(),
                                element,
                                mouse_x,
                                mouse_y,
                                factor,
                            );
                            app.set_event_handled();
                        }
                    }
                    Event::Unknown {
                        timestamp: _,
                        type_,
                    } => {
                        if *type_ == SDL_EventType::PINCH_UPDATE {
                            let pinch = unsafe { app.raw_event.pinch };
                            let mouse = app.event_pump.mouse_state();
                            zoom_graph(
                                pos.clone(),
                                zoom.clone(),
                                element,
                                &mouse.x(),
                                &mouse.y(),
                                pinch.scale,
                            );

                            app.set_event_handled();
                        }
                    }
                    _ => {}
                }
            }
        })
        .set_draw(move |app: &mut App, element: NodeId| {
            let element = app.tree[element].get();
            let mut atlas_txt = atlas_txt.borrow_mut();
            atlas_txt.set_color_mod(255, 255, 255);
            draw_tiled(
                &mut app.canvas,
                &atlas_txt,
                Some(FRect::new(0.0, 0.0, 128.0, 128.0)),
                1.0,
                Some(element.aabb),
            )
            .unwrap();
            app.canvas.set_clip_rect(Some(Rect::new(
                element.aabb.x as _,
                element.aabb.y as _,
                element.aabb.w as _,
                element.aabb.h as _,
            )));
            let zoom = zoom.borrow();
            let pos = pos.borrow();
            let x = element.aabb.x + element.aabb.w / 2.0 + pos.x;
            let y = element.aabb.y + element.aabb.h / 2.0 + pos.y;
            let length = NODE_LENGTH * *zoom;

            let nodes = nodes.borrow();
            for (id, node) in nodes.iter() {
                let node_x = x + node.x * *zoom;
                let node_y = y + node.y * *zoom;
                let (line_color, anim) = if let Some(selected) = *selected.borrow()
                    && *id == selected
                {
                    (LIGHT_BLUE, true)
                } else {
                    (
                        match node.state {
                            QuestState::Unavailable => DARK_PINK,
                            QuestState::Available => GRAY_PINK,
                            QuestState::Done => GREEN,
                        },
                        false,
                    )
                };
                for pre_id in &node.prerequisites {
                    if let Some(pre) = nodes.get(pre_id) {
                        let (line_color, anim) = if let Some(selected) = *selected.borrow()
                            && *pre_id == selected
                        {
                            (YELLOW, true)
                        } else {
                            (line_color, anim)
                        };
                        let color = FColor::from(line_color);
                        // atlas_txt.set_color_mod(line_color.r, line_color.g, line_color.b);
                        let size = length * 0.1;
                        let start = FPoint::new(
                            x + pre.x * *zoom + length / 2.0,
                            y + pre.y * *zoom + length / 2.0, // - h / 2.0,
                        );
                        let end = FPoint::new(
                            node_x + length / 2.0,
                            node_y + length / 2.0, /* - h / 2.0 */
                        );
                        let distance =
                            f32::sqrt((end.x - start.x).powf(2.0) + (end.y - start.y).powf(2.0));
                        let normalized_dir =
                            FPoint::new((end.x - start.x) / distance, (end.y - start.y) / distance);
                        let start_a = FPoint::new(
                            start.x + normalized_dir.y * size,
                            start.y - normalized_dir.x * size,
                        );
                        let start_b = FPoint::new(
                            start.x - normalized_dir.y * size,
                            start.y + normalized_dir.x * size,
                        );
                        let end_a = FPoint::new(
                            end.x + normalized_dir.y * size,
                            end.y - normalized_dir.x * size,
                        );
                        let end_b = FPoint::new(
                            end.x - normalized_dir.y * size,
                            end.y + normalized_dir.x * size,
                        );
                        // let angle = (end.x - start.x).atan2(end.y - start.y);
                        // app.canvas
                        //     .copy_ex(
                        //         &atlas_txt,
                        //         Some(FRect::new(0.0, 128.0, 26.0, 32.0)),
                        //         Some(FRect::new(start.x, start.y, distance, h)),
                        //         (end.y - start.y).atan2(end.x - start.x).to_degrees().into(),
                        //         Some(FPoint::new(0.0, 0.0)),
                        //         false,
                        //         false,
                        //     )
                        //     .unwrap();

                        fn lerp(a: FPoint, b: FPoint, p: f32) -> FPoint {
                            FPoint {
                                x: a.x + (b.x - a.x) * p,
                                y: a.y + (b.y - a.y) * p,
                            }
                        }

                        let mut progress = if anim {
                            (timer::ticks() as f32 / 1000.0).fract() * size * 2.0
                        } else {
                            0.0
                        };
                        let mut vertices = Vec::new();
                        let mut indices: Vec<u8> = Vec::new();
                        while progress < distance {
                            let p = progress / distance;
                            let p2 = (progress + size * 2.0) / distance;
                            progress += size * 2.0;
                            let l = vertices.len() as u8;
                            indices.extend_from_slice(&[l + 0, l + 1, l + 2, l + 1, l + 3, l + 2]);
                            vertices.extend_from_slice(&[
                                Vertex {
                                    position: lerp(start_a, end_a, p),
                                    color,
                                    tex_coord: FPoint::new(0.0, 128.0 / 512.0),
                                },
                                Vertex {
                                    position: lerp(start_b, end_b, p),
                                    color,
                                    tex_coord: FPoint::new(0.0, 192.0 / 512.0),
                                },
                                Vertex {
                                    position: lerp(start_a, end_a, p2),
                                    color,
                                    tex_coord: FPoint::new(64.0 / 512.0, 128.0 / 512.0),
                                },
                                Vertex {
                                    position: lerp(start_b, end_b, p2),
                                    color,
                                    tex_coord: FPoint::new(64.0 / 512.0, 192.0 / 512.0),
                                },
                            ]);
                        }
                        app.canvas
                            .render_geometry(&vertices, Some(&atlas_txt), &indices)
                            .unwrap();

                        // app.canvas.set_draw_color(color);
                        // app.canvas
                        //     .draw_line(
                        //         FPoint::new(
                        //             x + pre.x * *zoom + length / 2.0,
                        //             y + pre.y * *zoom + length / 2.0,
                        //         ),
                        //         FPoint::new(node_x + length / 2.0, node_y + length / 2.0),
                        //     )
                        //     .unwrap();
                    }
                }
            }

            for node in nodes.values() {
                let (fill, border) = match node.state {
                    QuestState::Unavailable => (DARK_GRAY, LIGHT_GRAY),
                    QuestState::Available => (LIGHT_GRAY, Color::WHITE),
                    QuestState::Done => (LIGHT_GRAY, GREEN),
                };

                let node_aabb = FRect::new(x + node.x * *zoom, y + node.y * *zoom, length, length);
                let fill_aabb = FRect::new(
                    node_aabb.x + length * 0.05,
                    node_aabb.y + length * 0.05,
                    length * 0.9,
                    length * 0.9,
                );

                app.canvas.set_draw_color(border);
                app.canvas.fill_rect(node_aabb).unwrap();

                app.canvas.set_draw_color(fill);
                app.canvas.fill_rect(fill_aabb).unwrap();
            }

            if let Some(id) = &*selected.borrow() {
                let node = nodes.get(id).unwrap();
                let node_aabb = FRect::new(x + node.x * *zoom, y + node.y * *zoom, length, length);
                app.canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
                app.canvas.set_draw_color(WHITE_OVERLAY);
                app.canvas.fill_rect(node_aabb).unwrap();
            }

            app.canvas.set_clip_rect(None);
        });
    graph
}

fn zoom_graph(
    pos: Rc<RefCell<FPoint>>,
    zoom: Rc<RefCell<f32>>,
    element: &mut Element<'_>,
    mouse_x: &f32,
    mouse_y: &f32,
    factor: f32,
) {
    {
        let pos = pos.clone();
        let zoom = zoom.clone();
        let mouse_x: &f32 = mouse_x;
        let mouse_y: &f32 = mouse_y;
        let mut pos = pos.borrow_mut();
        let mut zoom = zoom.borrow_mut();

        let mouse_x = *mouse_x - element.aabb.x - element.aabb.w / 2.0;
        let mouse_y = *mouse_y - element.aabb.y - element.aabb.h / 2.0;

        pos.x -= mouse_x;
        pos.y -= mouse_y;
        *zoom *= factor;
        pos.x *= factor;
        pos.y *= factor;
        pos.x += mouse_x;
        pos.y += mouse_y;
    };
}
