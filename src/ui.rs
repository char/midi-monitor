use crate::chord::{Chord, format_chord, identify_chord_in_context, roman_numeral};
use crate::note::{SpellingContext, is_white_key, scale_intervals};
use crate::{MidiMonitorParams, MidiMonitorParamsParamId as P, NOTE_COUNT, note_meter_id};
use truce::prelude::*;

const PIANO_LOW_NOTE: usize = 21;
const PIANO_HIGH_NOTE: usize = 108;
const PIANO_HEIGHT: f32 = 120.0;

const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(18, 18, 20);
const FOREGROUND: egui::Color32 = egui::Color32::from_rgb(238, 238, 240);
const MUTED: egui::Color32 = egui::Color32::from_rgb(150, 150, 156);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(238, 89, 175);
const OFF_ACCENT: egui::Color32 = egui::Color32::from_rgb(176, 103, 255);
const KEY_OUTLINE: egui::Color32 = egui::Color32::from_rgb(220, 220, 226);

pub fn visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = BACKGROUND;
    visuals.extreme_bg_color = egui::Color32::from_rgb(12, 12, 14);
    visuals.faint_bg_color = egui::Color32::from_rgb(28, 28, 31);
    visuals.override_text_color = Some(FOREGROUND);
    visuals.selection.bg_fill = ACCENT.linear_multiply(0.45);
    visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);

    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, MUTED);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(48, 48, 52);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, FOREGROUND);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(62, 58, 64);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, FOREGROUND);
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    visuals
}

pub fn draw_editor(ui: &mut egui::Ui, state: &PluginContext<MidiMonitorParams>) {
    let spelling = SpellingContext::new(state.root.value(), state.scale.value());
    let active_notes: Vec<_> = (0..NOTE_COUNT)
        .filter(|note| state.get_meter(note_meter_id(*note)) > 0.0)
        .collect();
    let root_override = state.chord_root.value().pitch_class();
    let chord = identify_chord_in_context(&active_notes, root_override, &spelling);

    ui.painter().rect_filled(ui.max_rect(), 0.0, BACKGROUND);
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);
    ui.style_mut()
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(11.0));
    ui.style_mut()
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(11.0));

    let top_height = (ui.available_height() - PIANO_HEIGHT - 24.0).max(0.0);
    egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(12, 12))
        .show(ui, |ui| {
            ui.set_min_height(top_height);
            top_bar(ui, state);
            ui.add_space(10.0);
            chord_panel(ui, chord.as_ref(), &spelling);
        });
    draw_piano(ui, state, &spelling);
}

fn top_bar(ui: &mut egui::Ui, state: &PluginContext<MidiMonitorParams>) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("charlotte's midi monitor").size(14.5).color(MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            param_dropdown(ui, state, P::Scale.as_u32());
            root_dropdown(ui, state);
        });
    });
}

fn chord_panel(ui: &mut egui::Ui, chord: Option<&Chord>, spelling: &SpellingContext) {
    let side_padding = ui.available_width() * 0.33;
    ui.horizontal(|ui| {
        ui.add_space(side_padding);
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(chord_name(chord, spelling))
                    .size(42.0)
                    .strong()
                    .color(FOREGROUND),
            );
            if let Some(chord) = chord
                && let Some(roman) = roman_numeral(chord, spelling.root_pitch_class, spelling.scale)
            {
                ui.add_space(2.0);
                ui.label(egui::RichText::new(roman).size(18.0).color(MUTED));
            }
        });
    });
}

fn draw_piano(ui: &mut egui::Ui, state: &PluginContext<MidiMonitorParams>, spelling: &SpellingContext) {
    let desired = egui::vec2(ui.available_width(), ui.available_height().min(PIANO_HEIGHT));
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let label_h = 22.0;
    let keys = egui::Rect::from_min_max(egui::pos2(rect.left(), rect.top() + label_h), rect.right_bottom());
    let white_count = (PIANO_LOW_NOTE..=PIANO_HIGH_NOTE)
        .filter(|note| is_white_key(note % 12))
        .count();
    let white_w = keys.width() / white_count as f32;
    let black_w = white_w * 0.66;
    let black_h = keys.height() * 0.62;
    let mut white_index = 0;

    painter.rect_filled(rect, 0.0, BACKGROUND);

    for note in PIANO_LOW_NOTE..=PIANO_HIGH_NOTE {
        let pitch_class = note % 12;
        if !is_white_key(pitch_class) {
            continue;
        }

        let x = keys.left() + white_index as f32 * white_w;
        let key_rect = egui::Rect::from_min_size(egui::pos2(x, keys.top()), egui::vec2(white_w, keys.height()));
        let velocity = state.get_meter(note_meter_id(note));
        painter.rect_filled(key_rect, 0.0, key_color(velocity, is_diatonic(pitch_class, spelling)));
        stroke_rect(&painter, key_rect, KEY_OUTLINE);
        draw_key_marks(
            &painter,
            white_top_edge_center(key_rect, note, black_w) + adjacent_label_shift(state, note, white_w),
            rect.top() + 11.0,
            note,
            velocity,
            spelling,
        );

        white_index += 1;
    }

    white_index = 0;
    for note in PIANO_LOW_NOTE..=PIANO_HIGH_NOTE {
        let pitch_class = note % 12;
        if is_white_key(pitch_class) {
            white_index += 1;
            continue;
        }

        let x =
            keys.left() + white_index as f32 * white_w + black_key_center_shift(pitch_class, black_w) - black_w / 2.0;
        let key_rect = egui::Rect::from_min_size(egui::pos2(x, keys.top()), egui::vec2(black_w, black_h));
        let velocity = state.get_meter(note_meter_id(note));
        painter.rect_filled(key_rect.expand(0.5), 0.0, BACKGROUND);
        painter.rect_filled(key_rect, 0.0, key_color(velocity, is_diatonic(pitch_class, spelling)));
        stroke_rect(&painter, key_rect, KEY_OUTLINE);
        draw_key_marks(
            &painter,
            key_rect.center().x + adjacent_label_shift(state, note, white_w),
            rect.top() + 11.0,
            note,
            velocity,
            spelling,
        );
    }
}

fn white_top_edge_center(key_rect: egui::Rect, note: usize, black_w: f32) -> f32 {
    let mut left = key_rect.left();
    let mut right = key_rect.right();

    if note > PIANO_LOW_NOTE {
        let pitch_class = (note - 1) % 12;
        if !is_white_key(pitch_class) {
            let black_center = key_rect.left() + black_key_center_shift(pitch_class, black_w);
            left = left.max(black_center + black_w / 2.0);
        }
    }

    if note < PIANO_HIGH_NOTE {
        let pitch_class = (note + 1) % 12;
        if !is_white_key(pitch_class) {
            let black_center = key_rect.right() + black_key_center_shift(pitch_class, black_w);
            right = right.min(black_center - black_w / 2.0);
        }
    }

    (left + right) / 2.0
}

fn black_key_center_shift(pitch_class: usize, black_w: f32) -> f32 {
    match pitch_class {
        1 => -black_w / 6.0,
        3 => black_w / 6.0,
        6 => -black_w / 4.0,
        8 => 0.0,
        10 => black_w / 4.0,
        _ => 0.0,
    }
}

fn adjacent_label_shift(state: &PluginContext<MidiMonitorParams>, note: usize, white_w: f32) -> f32 {
    let left_active = note > 0 && state.get_meter(note_meter_id(note - 1)) > 0.0;
    let right_active = note + 1 < NOTE_COUNT && state.get_meter(note_meter_id(note + 1)) > 0.0;

    match (left_active, right_active) {
        (true, false) => white_w * 0.18,
        (false, true) => -white_w * 0.18,
        _ => 0.0,
    }
}

fn draw_key_marks(
    painter: &egui::Painter,
    label_x: f32,
    label_y: f32,
    note: usize,
    velocity: f32,
    spelling: &SpellingContext,
) {
    let pitch_class = note % 12;
    if velocity > 0.0 {
        painter.text(
            egui::pos2(label_x, label_y),
            egui::Align2::CENTER_CENTER,
            spelling.pitch_name(pitch_class),
            egui::FontId::proportional(14.0),
            FOREGROUND,
        );
    }
}

fn stroke_rect(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    painter.line_segment([rect.left_top(), rect.right_top()], (1.0, color));
    painter.line_segment([rect.right_top(), rect.right_bottom()], (1.0, color));
    painter.line_segment([rect.right_bottom(), rect.left_bottom()], (1.0, color));
    painter.line_segment([rect.left_bottom(), rect.left_top()], (1.0, color));
}

fn key_color(velocity: f32, diatonic: bool) -> egui::Color32 {
    if velocity > 0.0 {
        let color = if diatonic { ACCENT } else { OFF_ACCENT };
        color.linear_multiply(0.55 + velocity * 0.45)
    } else {
        egui::Color32::TRANSPARENT
    }
}

fn is_diatonic(pitch_class: usize, spelling: &SpellingContext) -> bool {
    scale_intervals(spelling.scale)
        .iter()
        .any(|interval| (spelling.root_pitch_class + interval) % 12 == pitch_class)
}

fn chord_name(chord: Option<&Chord>, spelling: &SpellingContext) -> String {
    let Some(chord) = chord else {
        return String::new();
    };
    format_chord(chord, spelling)
}

fn root_dropdown(ui: &mut egui::Ui, state: &PluginContext<MidiMonitorParams>) {
    const ROOT_ROWS: &[&[(usize, &str)]] = &[
        &[(0, "C")],
        &[(1, "C#"), (2, "Db")],
        &[(3, "D")],
        &[(4, "D#"), (5, "Eb")],
        &[(6, "E")],
        &[(7, "F")],
        &[(8, "F#"), (9, "Gb")],
        &[(10, "G")],
        &[(11, "G#"), (12, "Ab")],
        &[(13, "A")],
        &[(14, "A#"), (15, "Bb")],
        &[(16, "B")],
    ];

    let id = P::Root.as_u32();
    let Some(info) = state.params().param_infos().into_iter().find(|info| info.id == id) else {
        ui.label("?");
        return;
    };

    let count = info.range.step_count_usize() + 1;
    let selected = state.format_param(id);

    egui::ComboBox::from_id_salt(("root_dropdown", id))
        .selected_text(selected.as_str())
        .width(80.0)
        .show_ui(ui, |ui| {
            let available = ui.available_width();
            let full_width = if available.is_finite() {
                available.max(0.0)
            } else {
                60.0
            };
            let height = ui.spacing().interact_size.y;

            for row in ROOT_ROWS {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let width = full_width / row.len() as f32;

                    for &(index, label) in *row {
                        let norm = truce::core::cast::discrete_norm(index, count);
                        if ui
                            .add_sized(
                                egui::vec2(width, height),
                                egui::Button::selectable(selected == label, label),
                            )
                            .clicked()
                        {
                            state.automate(id, norm);
                        }
                    }
                });
            }
        });
}

fn param_dropdown(ui: &mut egui::Ui, state: &PluginContext<MidiMonitorParams>, id: u32) {
    ui.horizontal(|ui| {
        let Some(info) = state.params().param_infos().into_iter().find(|info| info.id == id) else {
            ui.label("?");
            return;
        };
        let count = info.range.step_count_usize() + 1;
        let selected = state.format_param(id);

        egui::ComboBox::from_id_salt(("compact_dropdown", id))
            .selected_text(selected.as_str())
            .width(96.0)
            .show_ui(ui, |ui| {
                for i in 0..count {
                    let norm = truce::core::cast::discrete_norm(i, count);
                    let plain = info.range.denormalize(norm);
                    let text = state
                        .params()
                        .format_value(id, plain)
                        .unwrap_or_else(|| format!("{plain:.0}"));
                    if ui.selectable_label(text == selected, text).clicked() {
                        state.automate(id, norm);
                    }
                }
            });
    });
}
