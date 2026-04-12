use std::cell::RefCell;
use std::rc::Rc;

use gansui::App;
use gansui::AppError;
use gansui::draw::draw_tiled;
use gansui::element::Align;
use gansui::element::Element;
use gansui::element::Layout;
use gansui::element::Length;
use gansui::element::Padding;
use gansui::indextree::Arena;
use gansui::is_in;
use gansui::parley;
use gansui::parley::FontContext;
use gansui::parley::FontFamily;
use gansui::parley::LayoutContext;
use gansui::parley::OverflowWrap;
use gansui::parley::StyleProperty;
use gansui::sdl3;
use gansui::sdl3::image::LoadTexture;
use gansui::sdl3::mouse::MouseButton;
use gansui::sdl3::pixels::Color;
use gansui::sdl3::render::FRect;
use gansui::swash::scale::ScaleContext;
use gansui::text::rich_text_element;

pub mod graph;
use graph::generate_graph;

use crate::graph::load_graph;

const BLUE_GRAY: Color = Color::RGB(74, 85, 103);
const LIGHT_GRAY: Color = Color::RGB(132, 132, 132);
const GREEN: Color = Color::RGB(100, 220, 100);
const LIGHT_BLUE: Color = Color::RGB(0, 200, 200);
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

    let layout: parley::Layout<Color> = builder.build(text);

    rich_text_element(Rc::new(RefCell::new(layout)), scale_ctx, fit_line)
}

pub fn run() -> Result<(), AppError> {
    let font_ctx = Rc::new(RefCell::new(FontContext::new()));
    let layout_ctx = Rc::new(RefCell::new(LayoutContext::<Color>::new()));
    let scale_ctx = Rc::new(RefCell::new(ScaleContext::new()));

    let app = App::new()?;
    let mut atlas_txt = app
        .texture_creator
        .load_texture(format!("{ASSETS}/atlas.png"))?;
    atlas_txt.set_scale_mode(sdl3::render::ScaleMode::Nearest);

    let atlas_txt = Rc::new(RefCell::new(atlas_txt));
    let atlas_txt1 = atlas_txt.clone();

    let selected: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));

    let nodes = load_graph();
    let mut tree = Arena::new();

    fn text_bg<'a>(atlas_txt: Rc<RefCell<sdl3::render::Texture>>) -> Element<'a> {
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
            .with_padding(Padding::from(5.0))
    }

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
            .with_background_color(Color::RGBA(0, 0, 0, 100))
            .align_vertical(Align::Center)
            .align_horizontal(Align::Center)
            .on_mouse_down({
                // let overlay = update_overlay.clone();
                let selected = selected.clone();
                move |app, element, button, _clicks, _x, _y, _event| {
                    if button == MouseButton::Left {
                        app.set_event_handled();
                        app.tree[element].get_mut().layout = Layout::None;
                        // overlay.borrow_mut().on = false;
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

    let layers = Element::new()
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
        });

    let update_overlay = |app: &mut App| {
        let element = app.tree[overlay].get_mut();
        element.layout = Layout::Flex;

        description_box.remove_children(&mut app.tree);
        let nodes = nodes.borrow();
        let selected = selected.borrow();
        let node = nodes.get(&selected.unwrap()).unwrap();

        let text_title = app.tree.new_node(
            text_bg(atlas_txt1.clone())
                .align_horizontal(Align::Center)
                .with_padding(Padding::from(10.0))
                .with_height(Length::Fit {
                    min: 10.0,
                    max: f32::MAX,
                }),
        );
        text_title.append_value(
            my_text_element(
                node.title,
                scale_ctx.clone(),
                font_ctx.clone(),
                layout_ctx.clone(),
                true,
            )
            .with_background_color(Color::RGBA(0, 0, 0, 0)),
            &mut app.tree,
        );

        let text_description = app.tree.new_node(
            text_bg(atlas_txt1.clone())
                .with_padding(Padding::from(10.0))
                .with_height(Length::Grow {
                    min: 10.0,
                    max: f32::MAX,
                }),
        );
        text_description.append_value(
            my_text_element(
                node.desc,
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
    };
    let graph = generate_graph(atlas_txt, update_overlay, nodes.clone(), selected.clone());
    let layers = tree.new_node(layers);
    layers.append(overlay, &mut tree);
    layers.append_value(graph, &mut tree);

    app.run(layers, tree)
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
