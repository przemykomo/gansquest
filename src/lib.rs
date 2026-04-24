use std::cell::RefCell;
use std::rc::Rc;

use gansui::App;
use gansui::button::ButtonState;
use gansui::button::button;
use gansui::draw::draw_nine_patch;
use gansui::draw::draw_tiled;
use gansui::element::Align;
use gansui::element::Axis;
use gansui::element::Axis::Horizontal;
use gansui::element::Element;
use gansui::element::Layout;
use gansui::element::Length;
use gansui::element::Padding;
use gansui::indextree::Arena;
use gansui::indextree::NodeId;
use gansui::is_in;
use gansui::parley;
use gansui::parley::FontContext;
use gansui::parley::FontFamily;
use gansui::parley::LayoutContext;
use gansui::parley::OverflowWrap;
use gansui::parley::PlainEditor;
use gansui::parley::StyleProperty;
use gansui::sdl3;
use gansui::sdl3::image::LoadTexture;
use gansui::sdl3::messagebox::ButtonData;
use gansui::sdl3::messagebox::ClickedButton;
use gansui::sdl3::messagebox::MessageBoxButtonFlag;
use gansui::sdl3::messagebox::MessageBoxFlag;
use gansui::sdl3::messagebox::show_message_box;
use gansui::sdl3::mouse::MouseButton;
use gansui::sdl3::pixels::Color;
use gansui::sdl3::render::FRect;
use gansui::swash::scale::ScaleContext;
use gansui::text::rich_text_element;

pub mod graph;
use gansui::text::text_edit_element;
use graph::generate_graph;

use crate::graph::QuestNode;
use crate::graph::QuestWorld;
use crate::graph::load_graph;

const ACCEPT_TXT: FRect = FRect {
    x: 64.0,
    y: 128.0,
    w: 16.0,
    h: 16.0,
};

const CANCEL_TXT: FRect = FRect {
    x: 80.0,
    y: 128.0,
    w: 16.0,
    h: 16.0,
};

const EDIT_TXT: FRect = FRect {
    x: 112.0,
    y: 128.0,
    w: 16.0,
    h: 16.0,
};

const PLUS_TXT: FRect = FRect {
    x: 64.0,
    y: 144.0,
    w: 16.0,
    h: 16.0,
};

const TRASH_TXT: FRect = FRect {
    x: 80.0,
    y: 144.0,
    w: 16.0,
    h: 16.0,
};

const DARK_BLUE: Color = Color::RGBA(0, 0, 200, 255);
const LIGHT_BLUE: Color = Color::RGB(0, 200, 200);
// const BIT_LIGHT_BLUE: Color = Color::RGB(0, 100, 200);
//
// const DARK_GREEN: Color = Color::RGB(52, 103, 57);
// const GREEN_BUTTON: Color = Color::RGB(121, 174, 111);
// const LIGHT_GREEN: Color = Color::RGB(159, 203, 152);
//
// const DARK_RED: Color = Color::RGB(94, 0, 6);
// const RED_BUTTON: Color = Color::RGB(155, 15, 6);
// const LIGHT_RED: Color = Color::RGB(213, 62, 15);

const GREEN: Color = Color::RGB(100, 220, 100);
const BLUE_GRAY: Color = Color::RGB(74, 85, 103);
const LIGHT_GRAY: Color = Color::RGB(132, 132, 132);
const YELLOW: Color = Color::RGB(200, 200, 0);
const DARK_GRAY: Color = Color::RGB(87, 87, 87);
const GRAY_PINK: Color = Color::RGB(164, 137, 140);
const DARK_PINK: Color = Color::RGB(123, 110, 117);
const VERY_DARK_GRAY: Color = Color::RGB(24, 29, 33);
const WHITE_OVERLAY: Color = Color::RGBA(255, 255, 255, 100);
const ASSETS: &str = if cfg!(feature = "android") {
    "."
} else {
    "./assets"
};

pub fn my_text_element<'a>(
    text: &str,
    scale_ctx: Rc<RefCell<ScaleContext>>,
    font_ctx: Rc<RefCell<FontContext>>,
    layout_ctx: Rc<RefCell<LayoutContext<Color>>>,
    fit_line: bool,
) -> Element<'a> {
    let layout = build_text_layout(text, font_ctx, layout_ctx);

    rich_text_element(
        Rc::new(RefCell::new(layout)),
        scale_ctx,
        fit_line,
        Default::default(),
    )
}

fn build_text_layout(
    text: &str,
    font_ctx: Rc<RefCell<FontContext>>,
    layout_ctx: Rc<RefCell<LayoutContext<Color>>>,
) -> parley::Layout<Color> {
    let brush_style = StyleProperty::Brush(Color::WHITE);
    let font_family = FontFamily::parse("minecraft").unwrap();

    const DISPLAY_SCALE: f32 = 1.0;

    let mut layout_ctx = layout_ctx.borrow_mut();

    let mut font_ctx = font_ctx.borrow_mut();
    let mut builder = layout_ctx.ranged_builder(&mut font_ctx, text, DISPLAY_SCALE, true);

    builder.push_default(brush_style);
    builder.push_default(font_family);
    builder.push_default(StyleProperty::OverflowWrap(OverflowWrap::Anywhere));
    builder.push_default(StyleProperty::FontSize(32.0));

    builder.build(text)
}

fn tiled_bg<'a>(atlas_txt: Rc<RefCell<sdl3::render::Texture>>) -> Element<'a> {
    Element::new()
        .with_width(Length::Grow {
            min: 0.0,
            max: f32::MAX,
        })
        .set_draw(move |app, element| {
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
        })
        .align_horizontal(Align::Center)
        .with_padding(Padding::from(10.0))
}

fn done_editing_button(
    tree: &mut Arena<Element<'_>>,
    description_box: NodeId,
    world: Rc<RefCell<QuestWorld>>,
    selected: Rc<RefCell<Option<u32>>>,
    atlas_txt: Rc<RefCell<sdl3::render::Texture>>,
    edit_button: NodeId,
    editor_rc: Rc<RefCell<PlainEditor<Color>>>,
    scale_ctx: Rc<RefCell<ScaleContext>>,
    font_ctx: Rc<RefCell<FontContext>>,
    layout_ctx: Rc<RefCell<LayoutContext<Color>>>,
    accept: bool,
) -> NodeId {
    let button_state = Rc::new(RefCell::new(ButtonState::None));
    let element = button(button_state.clone(), {
        let atlas_txt = atlas_txt.clone();
        move |app, _accept_button| {
            description_box.remove_children(&mut app.tree);

            let mut world = world.borrow_mut();
            let selected = selected.borrow();
            let node = world.nodes.get_mut(&selected.unwrap()).unwrap();
            if accept {
                node.set_content(editor_rc.borrow().raw_text().to_owned());
            }

            let text_title = app.tree.new_node(
                tiled_bg(atlas_txt.clone())
                    .align_horizontal(Align::Center)
                    .with_padding(Padding::from(10.0))
                    .with_height(Length::Fit {
                        min: 10.0,
                        max: f32::MAX,
                    }),
            );
            text_title.append_value(
                my_text_element(
                    &node.content[..node.title_split],
                    scale_ctx.clone(),
                    font_ctx.clone(),
                    layout_ctx.clone(),
                    true,
                )
                .with_background_color(Color::RGBA(0, 0, 0, 0)),
                &mut app.tree,
            );

            let text_description =
                app.tree
                    .new_node(tiled_bg(atlas_txt.clone()).with_height(Length::Grow {
                        min: 10.0,
                        max: f32::MAX,
                    }));

            text_description.append_value(
                my_text_element(
                    if node.title_split == node.content.len() {
                        ""
                    } else {
                        &node.content[node.title_split + 1..]
                    },
                    scale_ctx.clone(),
                    font_ctx.clone(),
                    layout_ctx.clone(),
                    false,
                )
                .with_background_color(Color::RGBA(0, 0, 0, 0)),
                &mut app.tree,
            );
            description_box.append(text_title, &mut app.tree);
            description_box.append(text_description, &mut app.tree);
            text_description.append(edit_button, &mut app.tree);
        }
    })
    .with_width(Length::Fixed(64.0))
    .with_height(Length::Fixed(64.0))
    .set_draw(button_draw(
        atlas_txt,
        button_state,
        if accept { ACCEPT_TXT } else { CANCEL_TXT },
    ));
    // .set_draw({
    //     let atlas_txt = atlas_txt.clone();
    //     move |app, element| {
    //         let color = match *accept_button_state.borrow() {
    //             ButtonState::None => none,
    //             ButtonState::Hover => hover,
    //             ButtonState::Pressed => pressed,
    //         };
    //
    //         let element = app.tree[element].get();
    //         let mut atlas_txt = atlas_txt.borrow_mut();
    //         atlas_txt.set_color_mod(color.r, color.g, color.b);
    //         app.canvas
    //             .copy(
    //                 &atlas_txt,
    //                 Some(if accept { ACCEPT_TXT } else { CANCEL_TXT }),
    //                 Some(element.aabb),
    //             )
    //             .unwrap();
    //     }
    // }),
    tree.new_node(element)
}

pub fn run() -> anyhow::Result<()> {
    let mut save_directory = if !cfg!(debug_assertions) || cfg!(feature = "android") {
        sdl3::filesystem::get_pref_path("przemyk", "gansquest").unwrap()
    } else {
        "./save/".into()
    };

    let font_ctx = Rc::new(RefCell::new(FontContext::new()));
    let layout_ctx = Rc::new(RefCell::new(LayoutContext::<Color>::new()));
    let scale_ctx = Rc::new(RefCell::new(ScaleContext::new()));

    let app = App::new(VERY_DARK_GRAY)?;
    let mut atlas_txt = app
        .texture_creator
        .load_texture(format!("{ASSETS}/atlas.png"))?;
    atlas_txt.set_scale_mode(sdl3::render::ScaleMode::Nearest);

    let atlas_txt = Rc::new(RefCell::new(atlas_txt));

    let selected: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));

    save_directory.push("myworld/");
    let world = load_graph(save_directory);
    let mut tree = Arena::new();

    let description_box = tree.new_node(
        Element::new()
            .with_width(Length::Fit {
                min: 820.0,
                max: f32::MAX,
            })
            .with_height(Length::Fit {
                min: 350.0,
                max: f32::MAX,
            })
            .with_padding(Padding::from(4.0))
            .with_spacing(4.0)
            .on_mouse_down({
                move |app, element, button, _clicks, x, y, _event| {
                    let element = app.tree[element].get();
                    if button == MouseButton::Left && is_in(x, y, &element.aabb) {
                        app.set_event_handled();
                    }
                }
            })
            .with_background_color(VERY_DARK_GRAY),
    );

    let overlay = tree.new_node(
        Element::new()
            .with_width(Length::Grow {
                min: 0.0,
                max: f32::MAX,
            })
            .with_height(Length::Grow {
                min: 0.0,
                max: f32::MAX,
            })
            .with_layout(Layout::None)
            .with_background_color(Color::RGBA(0, 0, 0, 70))
            .align_vertical(Align::Center)
            .align_horizontal(Align::Center)
            .on_mouse_down({
                let selected = selected.clone();
                move |app, element, button, _clicks, _x, _y, _event| {
                    if button == MouseButton::Left {
                        app.set_event_handled();
                        app.tree[element].get_mut().layout = Layout::None;
                        *selected.borrow_mut() = None;
                    }
                }
            })
            .on_mouse_motion(
                move |app,
                      _element,
                      _mouse_x,
                      _mouse_y,
                      _diff_x,
                      _diff_y,
                      _mousestate,
                      _event,
                      _update_selection| {
                    app.set_event_handled();
                },
            )
            .set_draw(move |app, element| {
                let element = app.tree[element].get();
                app.canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
                app.canvas.set_draw_color(element.background_color);
                app.canvas.fill_rect(element.aabb).unwrap();
            }),
    );
    overlay.append(description_box, &mut tree);

    let edit_button_state = Rc::new(RefCell::new(ButtonState::None));
    let edit_button = tree.new_node(edit_button(
        description_box,
        edit_button_state.clone(),
        world.clone(),
        selected.clone(),
        atlas_txt.clone(),
        scale_ctx.clone(),
        font_ctx.clone(),
        layout_ctx.clone(),
        overlay,
    ));

    let update_overlay = {
        let atlas_txt = atlas_txt.clone();
        let world = world.clone();
        let selected = selected.clone();
        let scale_ctx = scale_ctx.clone();
        let font_ctx = font_ctx.clone();
        let layout_ctx = layout_ctx.clone();
        move |app: &mut App| {
            let element = app.tree[overlay].get_mut();
            element.layout = Layout::Flex;

            edit_button.detach(&mut app.tree);
            description_box.remove_children(&mut app.tree);
            let world = world.borrow();
            let selected = selected.borrow();
            let node = world.nodes.get(&selected.unwrap()).unwrap();

            let text_title = app.tree.new_node(
                tiled_bg(atlas_txt.clone())
                    .align_horizontal(Align::Center)
                    .with_height(Length::Fit {
                        min: 10.0,
                        max: f32::MAX,
                    }),
            );
            text_title.append_value(
                my_text_element(
                    &node.content[..node.title_split],
                    scale_ctx.clone(),
                    font_ctx.clone(),
                    layout_ctx.clone(),
                    true,
                )
                .with_background_color(Color::RGBA(0, 0, 0, 0)),
                &mut app.tree,
            );

            let text_description =
                app.tree
                    .new_node(tiled_bg(atlas_txt.clone()).with_height(Length::Grow {
                        min: 10.0,
                        max: f32::MAX,
                    }));
            text_description.append_value(
                my_text_element(
                    if node.title_split == node.content.len() {
                        ""
                    } else {
                        &node.content[node.title_split + 1..]
                    },
                    scale_ctx.clone(),
                    font_ctx.clone(),
                    layout_ctx.clone(),
                    false,
                )
                .with_background_color(Color::RGBA(0, 0, 0, 0)),
                &mut app.tree,
            );

            description_box.append(text_title, &mut app.tree);
            description_box.append(text_description, &mut app.tree);
            text_description.append(edit_button, &mut app.tree);
        }
    };
    let graph = tree.new_node(generate_graph(
        atlas_txt.clone(),
        update_overlay,
        world.clone(),
        selected.clone(),
    ));
    let tooltip_text = Rc::new(RefCell::new(build_text_layout(
        "tooltip",
        font_ctx.clone(),
        layout_ctx.clone(),
    )));
    let cached_width: Rc<RefCell<Option<f32>>> = Default::default();
    let tooltip_text_node = tree.new_node(rich_text_element(
        tooltip_text.clone(),
        scale_ctx,
        true,
        cached_width.clone(),
    ));
    let tooltip = tree.new_node(
        Element::new()
            .with_width(Length::Fit {
                min: 20.0,
                max: f32::MAX,
            })
            .with_height(Length::Fit {
                min: 10.0,
                max: f32::MAX,
            })
            .with_padding(Padding::from(10.0))
            .with_layout(Layout::Custom)
            .set_draw({
                let tooltip_text = tooltip_text.clone();
                let selected = selected.clone();
                let world = world.clone();
                let font_ctx = font_ctx.clone();
                let layout_ctx = layout_ctx.clone();
                let cached_width = cached_width.clone();
                let atlas_txt = atlas_txt.clone();
                let mut prev = None;
                move |app, element| {
                    let selected = selected.borrow();
                    if selected.is_none() || app.tree[overlay].get().layout != Layout::None {
                        prev = None;
                        tooltip_text_node.detach(&mut app.tree);
                        return;
                    }

                    if tooltip_text_node.parent(&app.tree).is_none() {
                        element.append(tooltip_text_node, &mut app.tree);
                    }

                    if *selected != prev {
                        prev = *selected;
                        let Some(selected) = *selected else {
                            tooltip_text_node.detach(&mut app.tree);
                            return;
                        };
                        let world = world.borrow();
                        let node = world.nodes.get(&selected).unwrap();
                        let text = &node.content[..node.title_split];
                        let layout = build_text_layout(text, font_ctx.clone(), layout_ctx.clone());
                        let width = layout.calculate_content_widths();
                        *cached_width.borrow_mut() = None;
                        let child = app.tree[element].first_child().unwrap();
                        app.tree[child].get_mut().width = Length::Fit {
                            min: width.max,
                            max: f32::MAX,
                        };
                        *tooltip_text.borrow_mut() = layout;
                    }

                    app.tree[element].get_mut().layout = Layout::Flex;

                    app.fit_size(element, Axis::Horizontal);
                    app.grow_shrink_size_wrap(element, Axis::Horizontal);
                    app.fit_size(element, Axis::Vertical);
                    app.grow_shrink_size_wrap(element, Axis::Vertical);

                    let e = app.tree[element].get_mut();
                    let mouse = app.event_pump.mouse_state();
                    let size = app.canvas.output_size().unwrap();
                    e.aabb.x = (mouse.x() + 20.0).max(0.0);
                    if e.aabb.x + e.aabb.w > size.0 as f32 {
                        e.aabb.x = mouse.x() - e.aabb.w - 20.0;
                    }
                    e.aabb.y = (mouse.y() - 40.0).max(0.0).min(size.1 as f32 - e.aabb.h);

                    app.position_elements(element);

                    let e = app.tree[element].get_mut();
                    e.layout = Layout::Custom;

                    draw_nine_patch(
                        &mut app.canvas,
                        &atlas_txt.borrow(),
                        Some(FRect::new(0.0, 192.0, 8.0, 8.0)),
                        e.aabb,
                        4.0,
                    )
                    .unwrap();
                }
            }),
    );
    tooltip.append(tooltip_text_node, &mut tree);
    let layers = tree.new_node(
        Element::new()
            .with_width(Length::Grow {
                min: 0.0,
                max: f32::MAX,
            })
            .with_height(Length::Grow {
                min: 0.0,
                max: f32::MAX,
            })
            .with_layout(Layout::Custom)
            .set_draw(|app, element| {
                let element = &app.tree[element];
                let aabb = element.get().aabb;
                let mut child_id = element.first_child();

                while let Some(c) = child_id {
                    app.flex_layout(c, aabb);
                    child_id = app.tree[c].next_sibling();
                }
            }),
    );
    let tooltip_layer = tree.new_node(Element::new());
    tooltip_layer.append(tooltip, &mut tree);
    layers.append(tooltip_layer, &mut tree);
    layers.append(overlay, &mut tree);
    let main_layer = Element::new()
        .with_width(Length::Grow {
            min: 0.0,
            max: f32::MAX,
        })
        .with_height(Length::Grow {
            min: 0.0,
            max: f32::MAX,
        })
        .with_padding(10.0.into())
        .with_spacing(10.0);

    #[cfg(not(feature = "android"))]
    let main_layer = main_layer.with_layout_axis(Axis::Horizontal);

    let main_layer = tree.new_node(main_layer);
    main_layer.append(graph, &mut tree);
    let sidebar = Element::new()
        .with_padding(4.0.into())
        .with_spacing(4.0)
        .set_draw({
            let atlas_txt = atlas_txt.clone();
            move |app, element| {
                let element = app.tree[element].get();
                let mut atlas_txt = atlas_txt.borrow_mut();
                atlas_txt.set_color_mod(200, 200, 200);
                draw_tiled(
                    &mut app.canvas,
                    &atlas_txt,
                    Some(FRect::new(0.0, 0.0, 128.0, 128.0)),
                    1.0,
                    Some(element.aabb),
                )
                .unwrap();
            }
        });

    #[cfg(feature = "android")]
    let sidebar = sidebar
        .with_height(Length::Fit {
            min: 0.0,
            max: f32::MAX,
        })
        .with_width(Length::Grow {
            min: 0.0,
            max: f32::MAX,
        })
        .align_vertical(Align::Center);

    #[cfg(not(feature = "android"))]
    let sidebar = sidebar
        .with_height(Length::Grow {
            min: 0.0,
            max: f32::MAX,
        })
        .with_width(Length::Fit {
            min: 0.0,
            max: f32::MAX,
        })
        .align_horizontal(Align::Center);

    let sidebar = tree.new_node(sidebar);

    let button_state: Rc<RefCell<ButtonState>> = Default::default();
    sidebar.append_value(
        button(button_state.clone(), {
            let world = world.clone();
            move |_app, _element| {
                let mut world = world.borrow_mut();
                world.add(QuestNode::new(
                    0.0, //TODO
                    0.0,
                    vec![],
                    false,
                    // graph::QuestState::Available,
                    "New Quest".to_owned(),
                ));
            }
        })
        .with_width(Length::Fixed(64.0))
        .with_height(Length::Fixed(64.0))
        .set_draw(button_draw(atlas_txt, button_state, PLUS_TXT)),
        &mut tree,
    );
    main_layer.append(sidebar, &mut tree);
    layers.append(main_layer, &mut tree);

    app.run(layers, tree)?;
    world.borrow().save()?;
    Ok(())
}

fn edit_button<'a>(
    description_box: NodeId,
    edit_button_state: Rc<RefCell<ButtonState>>,
    world: Rc<RefCell<QuestWorld>>,
    selected: Rc<RefCell<Option<u32>>>,
    atlas_txt: Rc<RefCell<sdl3::render::Texture>>,
    scale_ctx: Rc<RefCell<ScaleContext>>,
    font_ctx: Rc<RefCell<FontContext>>,
    layout_ctx: Rc<RefCell<LayoutContext<Color>>>,
    overlay: NodeId,
) -> Element<'a> {
    button(edit_button_state.clone(), {
        let atlas_txt = atlas_txt.clone();
        move |app, edit_button| {
            edit_button.detach(&mut app.tree);
            description_box.remove_children(&mut app.tree);

            let w = world.borrow();
            let s = selected.borrow();
            let node = w.nodes.get(&s.unwrap()).unwrap();

            let edit_text_description = app.tree.new_node(
                tiled_bg(atlas_txt.clone())
                    .with_height(Length::Grow {
                        min: 10.0,
                        max: f32::MAX,
                    })
                    .with_spacing(10.0),
            );

            let font = FontFamily::parse("minecraft").unwrap();
            let mut editor = PlainEditor::<Color>::new(32.0);
            editor.edit_styles().insert(font.into());
            let editor = Rc::new(RefCell::new(editor));
            edit_text_description.append_value(
                text_edit_element(
                    scale_ctx.clone(),
                    font_ctx.clone(),
                    layout_ctx.clone(),
                    app,
                    &node.content,
                    editor.clone(),
                )
                .with_background_color(Color::RGBA(0, 0, 0, 0)),
                &mut app.tree,
            );

            let accept_button = done_editing_button(
                &mut app.tree,
                description_box,
                world.clone(),
                selected.clone(),
                atlas_txt.clone(),
                edit_button,
                editor.clone(),
                scale_ctx.clone(),
                font_ctx.clone(),
                layout_ctx.clone(),
                true,
            );

            let discard_button = done_editing_button(
                &mut app.tree,
                description_box,
                world.clone(),
                selected.clone(),
                atlas_txt.clone(),
                edit_button,
                editor.clone(),
                scale_ctx.clone(),
                font_ctx.clone(),
                layout_ctx.clone(),
                false,
            );

            let delete_button = app.tree.new_node({
                let world = world.clone();
                let selected = selected.clone();
                let button_state: Rc<RefCell<ButtonState>> = Default::default();
                button(button_state.clone(), move |app, _element| {
                    let msgbox_buttons = [
                        ButtonData {
                            flags: MessageBoxButtonFlag::NOTHING,
                            button_id: 0,
                            text: "DELETE",
                        },
                        ButtonData {
                            flags: MessageBoxButtonFlag::ESCAPEKEY_DEFAULT
                                | MessageBoxButtonFlag::RETURNKEY_DEFAULT,
                            button_id: 1,
                            text: "CANCEL",
                        },
                    ];
                    let clicked = show_message_box(
                        MessageBoxFlag::WARNING,
                        &msgbox_buttons,
                        "Delete Node",
                        "Are you sure?",
                        app.canvas.window(),
                        None,
                    );

                    let Ok(ClickedButton::CustomButton(b)) = clicked else {
                        return;
                    };

                    if b.button_id == 0 {
                        let mut selected = selected.borrow_mut();
                        if let Some(s) = **&selected {
                            let mut world = world.borrow_mut();
                            world.nodes.remove(&s);
                            app.set_event_handled();
                            app.tree[overlay].get_mut().layout = Layout::None;
                            *selected = None;
                        }
                    }
                })
                .with_width(Length::Fixed(64.0))
                .with_height(Length::Fixed(64.0))
                .set_draw(button_draw(atlas_txt.clone(), button_state, TRASH_TXT))
            });

            description_box.append(edit_text_description, &mut app.tree);

            let options_box = app.tree.new_node(
                Element::new()
                    .with_width(Length::Grow {
                        min: 0.0,
                        max: f32::MAX,
                    })
                    .with_height(Length::Fit {
                        min: 0.0,
                        max: f32::MAX,
                    })
                    .with_layout_axis(Horizontal)
                    .align_horizontal(Align::Center)
                    .with_spacing(64.0),
            );

            options_box.append(accept_button, &mut app.tree);
            options_box.append(discard_button, &mut app.tree);
            options_box.append(delete_button, &mut app.tree);

            // I don't think this filler should be neccessary, but text edit doesn't want to grow
            // for some reason
            edit_text_description.append_value(
                Element::new().with_height(Length::Grow {
                    min: 0.0,
                    max: f32::MAX,
                }),
                &mut app.tree,
            );
            edit_text_description.append(options_box, &mut app.tree);
        }
    })
    .with_width(Length::Fixed(64.0))
    .with_height(Length::Fixed(64.0))
    .with_background_color(DARK_BLUE)
    .set_draw(button_draw(atlas_txt, edit_button_state, EDIT_TXT))
}

fn button_draw(
    atlas_txt: Rc<RefCell<sdl3::render::Texture>>,
    button_state: Rc<RefCell<ButtonState>>,
    txt: FRect,
) -> impl FnMut(&mut App<'_>, NodeId) {
    move |app, element| {
        let color = match *button_state.borrow() {
            ButtonState::None => Color {
                r: 190,
                g: 190,
                b: 190,
                a: 255,
            },
            ButtonState::Hover => Color {
                r: 210,
                g: 210,
                b: 210,
                a: 255,
            },
            ButtonState::Pressed => Color::WHITE,
        };

        let element = app.tree[element].get();
        let mut atlas_txt = atlas_txt.borrow_mut();
        atlas_txt.set_color_mod(color.r, color.g, color.b);
        app.canvas
            .copy(&atlas_txt, Some(txt), Some(element.aabb))
            .unwrap();
    }
}

#[cfg(feature = "android")]
use std::ffi::c_int;

#[cfg(feature = "android")]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub fn SDL_main() -> c_int {
    std::panic::set_hook(Box::new(|a| {
        if let Some(message) = a.payload_as_str() {
            let _ = sdl3::messagebox::show_simple_message_box(
                sdl3::messagebox::MessageBoxFlag::ERROR,
                "Panic!",
                message,
                None,
            );
        }
    }));
    run().unwrap();
    0
}
