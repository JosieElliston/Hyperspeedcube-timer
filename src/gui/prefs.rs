use egui::NumExt;

use super::util::{self, ResponseExt};
use crate::app::App;
use crate::preferences::DEFAULT_PREFS;
use crate::puzzle::PuzzleTypeTrait;
use crate::serde_impl::hex_color;

pub fn build(ui: &mut egui::Ui, app: &mut App) {
    ui.spacing_mut().interact_size.x *= 1.5;
    ui.style_mut().wrap = Some(false);

    ui.heading("Preferences");
    ui.separator();
    egui::ScrollArea::new([false, true]).show(ui, |ui| {
        ui.collapsing("Colors", |ui| build_colors_section(ui, app));
        ui.collapsing("Graphics", |ui| build_graphics_section(ui, app));
        ui.collapsing("View", |ui| build_view_section(ui, app));
        ui.collapsing("Interaction", |ui| {
            build_interaction_section(ui, app);

            ui.separator();

            ui.strong("Keybinds");
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    if ui.button("Edit general keybinds").clicked() {
                        super::Window::GeneralKeybinds.toggle(ui.ctx());
                    }
                    if ui.button("Edit puzzle keybinds").clicked() {
                        super::Window::PuzzleKeybinds.toggle(ui.ctx());
                    }
                },
            )
        });
    });
}

macro_rules! resettable {
    (
        $label:expr,
        ($prefs:ident $($prefs_tok:tt)*),
        $make_widget:expr $(,)?
    ) => {
        resettable!($label, "{}", ($prefs $($prefs_tok)*), $make_widget)
    };
    (
        $label:expr,
        $format_str:tt,
        ($prefs:ident $($prefs_tok:tt)*),
        $make_widget:expr $(,)?
    ) => {
        resettable!($label, |x| format!($format_str, x), ($prefs $($prefs_tok)*), $make_widget)
    };
    (
        $label:expr,
        $format_fn:expr,
        ($prefs:ident $($prefs_tok:tt)*),
        $make_widget:expr $(,)?
    ) => {{
        let value = &mut $prefs $($prefs_tok)*;
        let reset_value = &crate::preferences::DEFAULT_PREFS $($prefs_tok)*;
        #[allow(clippy::redundant_closure_call)]
        let reset_value_str = ($format_fn)(reset_value);
        crate::gui::util::WidgetWithReset {
            label: $label,
            value,
            reset_value: reset_value.clone(),
            reset_value_str,
            make_widget: $make_widget,
        }
    }};
}

fn build_colors_section(ui: &mut egui::Ui, app: &mut App) {
    let puzzle_type = app.puzzle.ty();
    let prefs = &mut app.prefs;

    let mut changed = false;

    // Opacity
    let r = ui.add(resettable!(
        "Sticker opacity",
        |x| format!("{:.0}%", x * 100.0),
        (prefs.colors.sticker_opacity),
        util::make_percent_drag_value,
    ));
    changed |= r.changed();
    let r = ui
        .add(resettable!(
            "Hidden opacity",
            |x| format!("{:.0}%", x * 100.0),
            (prefs.colors.hidden_opacity),
            util::make_percent_drag_value,
        ))
        .on_hover_explanation(
            "",
            "Opacity of hidden stickers (multiplied \
             by base sticker opacity)",
        );
    changed |= r.changed();

    ui.separator();

    // Special colors
    let r = ui.add(resettable!(
        "Background",
        hex_color::to_str,
        (prefs.colors.background),
        |value| |ui: &mut egui::Ui| ui.color_edit_button_srgba(value),
    ));
    changed |= r.changed();
    let r = ui.add(resettable!(
        "Outline",
        hex_color::to_str,
        (prefs.colors.outline),
        |value| |ui: &mut egui::Ui| ui.color_edit_button_srgba(value),
    ));
    changed |= r.changed();

    ui.separator();

    // Sticker colors
    for &face in puzzle_type.faces() {
        let r = ui.add(resettable!(
            face.name(),
            hex_color::to_str,
            (prefs.colors[face]),
            |value| |ui: &mut egui::Ui| ui.color_edit_button_srgba(value),
        ));
        changed |= r.changed();
    }

    ui.separator();

    // Blindfold colors
    let r = ui.add(resettable!(
        "Blindfolded stickers",
        hex_color::to_str,
        (prefs.colors.blind_face),
        |value| |ui: &mut egui::Ui| ui.color_edit_button_srgba(value),
    ));
    changed |= r.changed();
    let r = ui.add(util::CheckboxWithReset {
        label: "Blindfold mode",
        value: &mut prefs.colors.blindfold,
        reset_value: DEFAULT_PREFS.colors.blindfold,
    });
    changed |= r.changed();

    prefs.needs_save |= changed;
    app.wants_repaint |= changed;
}
fn build_graphics_section(ui: &mut egui::Ui, app: &mut App) {
    let prefs = &mut app.prefs;

    // FPS limit
    let r = ui
        .add(resettable!("FPS limit", (prefs.gfx.fps), |value| {
            egui::DragValue::new(value)
                .clamp_range(5..=255_u32)
                .speed(0.5)
        }))
        .on_hover_explanation("Frames Per Second", "");
    prefs.needs_save |= r.changed();

    // MSAA
    let r = ui
        .add(util::CheckboxWithReset {
            label: "MSAA",
            value: &mut prefs.gfx.msaa,
            reset_value: DEFAULT_PREFS.gfx.msaa,
        })
        .on_hover_explanation(
            "Multisample Anti-Aliasing",
            "Makes edges less jagged, \
             but may worsen performance.",
        );
    prefs.needs_save |= r.changed();
    app.wants_repaint |= r.changed();
}
fn build_view_section(ui: &mut egui::Ui, app: &mut App) {
    let puzzle_type = app.puzzle.ty();
    let prefs = &mut app.prefs;

    let mut changed = false;

    ui.strong("View angle");
    // Pitch
    let r = ui.add(resettable!(
        "Pitch",
        "{}°",
        (prefs.view[puzzle_type].pitch),
        |value| util::make_degrees_drag_value(value).clamp_range(-90.0..=90.0),
    ));
    changed |= r.changed();
    // Yaw
    let r = ui.add(resettable!(
        "Yaw",
        "{}°",
        (prefs.view[puzzle_type].yaw),
        |value| util::make_degrees_drag_value(value).clamp_range(-45.0..=45.0),
    ));
    changed |= r.changed();

    ui.separator();
    ui.strong("Projection");
    // Scale
    let r = ui.add(resettable!(
        "Scale",
        (prefs.view[puzzle_type].scale),
        |value| {
            let speed = *value / 100.0; // logarithmic speed
            egui::DragValue::new(value)
                .fixed_decimals(2)
                .clamp_range(0.1..=5.0_f32)
                .speed(speed)
        },
    ));
    changed |= r.changed();
    // 4D FOV
    let r = ui.add(resettable!(
        "4D FOV",
        "{}°",
        (prefs.view[puzzle_type].fov_4d),
        |value| {
            util::make_degrees_drag_value(value)
                .clamp_range(1.0..=120.0)
                .speed(0.5)
        },
    ));
    changed |= r.changed();
    // 3D FOV
    let r = ui.add(resettable!(
        "3D FOV",
        "{}°",
        (prefs.view[puzzle_type].fov_3d),
        |value| {
            util::make_degrees_drag_value(value)
                .clamp_range(-120.0..=120.0)
                .speed(0.5)
        },
    ));
    changed |= r.changed();

    ui.separator();

    ui.strong("Geometry");
    // Face spacing
    let r = ui.add(resettable!(
        "Face spacing",
        (prefs.view[puzzle_type].face_spacing),
        |value| {
            egui::DragValue::new(value)
                .fixed_decimals(2)
                .clamp_range(0.0..=0.9_f32)
                .speed(0.005)
        },
    ));
    changed |= r.changed();
    // Sticker spacing
    let r = ui.add(resettable!(
        "Sticker spacing",
        (prefs.view[puzzle_type].sticker_spacing),
        |value| {
            egui::DragValue::new(value)
                .fixed_decimals(2)
                .clamp_range(0.0..=0.9_f32)
                .speed(0.005)
        },
    ));
    changed |= r.changed();
    // Outline thickness
    let r = ui.add(resettable!(
        "Outline thickness",
        (prefs.view[puzzle_type].outline_thickness),
        |value| {
            egui::DragValue::new(value)
                .fixed_decimals(1)
                .clamp_range(0.0..=5.0_f32)
                .speed(0.01)
        },
    ));
    changed |= r.changed();

    ui.separator();

    ui.strong("Lighting");
    // Pitch
    let r = ui.add(resettable!(
        "Pitch",
        "{}°",
        (prefs.view[puzzle_type].light_pitch),
        |value| util::make_degrees_drag_value(value).clamp_range(-90.0..=90.0),
    ));
    changed |= r.changed();
    // Yaw
    let r = ui.add(resettable!(
        "Yaw",
        "{}°",
        (prefs.view[puzzle_type].light_yaw),
        |value| util::make_degrees_drag_value(value).clamp_range(-180.0..=180.0),
    ));
    changed |= r.changed();
    // Intensity
    let r = ui.add(resettable!(
        "Intensity",
        |x| format!("{:.0}%", x * 100.0),
        (prefs.view[puzzle_type].light_intensity),
        util::make_percent_drag_value,
    ));
    changed |= r.changed();

    prefs.needs_save |= changed;
    app.wants_repaint |= changed;
}
fn build_interaction_section(ui: &mut egui::Ui, app: &mut App) {
    let prefs = &mut app.prefs;

    let mut changed = false;

    ui.strong("Twist speed");
    let r = ui.add(resettable!(
        "Twist duration",
        (prefs.interaction.twist_duration),
        |value| {
            let speed = value.at_least(0.1) / 100.0; // logarithmic speed
            egui::DragValue::new(value)
                .fixed_decimals(2)
                .clamp_range(0.0..=5.0_f32)
                .speed(speed)
        },
    ));
    changed |= r.changed();
    let r = ui
        .add(util::CheckboxWithReset {
            label: "Dynamic twist speed",
            value: &mut prefs.interaction.dynamic_twist_speed,
            reset_value: DEFAULT_PREFS.interaction.dynamic_twist_speed,
        })
        .on_hover_explanation(
            "",
            "When enabled, the puzzle twists faster when \
             many moves are queued up. When all queued \
             moves are complete, the twist speed resets.",
        );
    changed |= r.changed();

    prefs.needs_save |= changed;
}