use crate::app::{App, CursorReadout, PreparedCurve};
use crate::plot;
use egui::epaint::Shadow;
use egui::{self, Color32, FontDefinitions, Margin, RichText, Vec2b};
use egui_plot::{Legend, Line, Plot, PlotBounds, PlotPoint, PlotPoints, PlotUi, Points, Text};

const ACCENT_COLOR: Color32 = Color32::from_rgb(80, 145, 255);

const NOTATIONS: &[(&str, &str, &str)] = &[
    ("π", "pi", "pi"),
    ("√", "sqrt()", "square root"),
    ("sin", "sin()", "sine"),
    ("cos", "cos()", "cosine"),
    ("tan", "tan()", "tangent"),
    ("ln", "ln()", "natural log"),
    ("|x|", "abs()", "absolute value"),
    ("^2", "^2", "square"),
    ("^3", "^3", "cube"),
];

pub fn install_fonts(ctx: &egui::Context) {
    ctx.set_fonts(FontDefinitions::default());
}

pub fn install_visuals(ctx: &egui::Context, dark: bool) {
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    let rounding = egui::Rounding::same(14.0);
    visuals.widgets.noninteractive.rounding = rounding;
    visuals.widgets.inactive.rounding = rounding;
    visuals.widgets.hovered.rounding = rounding;
    visuals.widgets.active.rounding = rounding;
    visuals.widgets.open.rounding = rounding;

    visuals.window_rounding = egui::Rounding::same(18.0);
    visuals.window_shadow = Shadow {
        offset: egui::vec2(0.0, 8.0),
        blur: 24.0,
        spread: 0.0,
        color: Color32::from_black_alpha(40),
    };

    visuals.selection.bg_fill = ACCENT_COLOR;
    visuals.selection.stroke.color = ACCENT_COLOR;
    visuals.hyperlink_color = ACCENT_COLOR;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(12.0, 10.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.window_margin = Margin::same(12.0);
    ctx.set_style(style);
}

pub fn top_bar(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.heading("Rust Native Graphing Calculator");
        ui.add_space(16.0);
        let mut range_changed = false;
        ui.label("x min");
        if ui
            .add(egui::DragValue::new(&mut app.x_min).speed(0.25))
            .changed()
        {
            range_changed = true;
        }

        ui.label("x max");
        if ui
            .add(egui::DragValue::new(&mut app.x_max).speed(0.25))
            .changed()
        {
            range_changed = true;
        }

        ui.label("samples");
        ui.add(
            egui::DragValue::new(&mut app.samples)
                .range(plot::MIN_SAMPLES as f64..=plot::MAX_SAMPLES as f64)
                .speed(64.0),
        );

        if range_changed {
            app.request_view_refresh();
        }

        ui.toggle_value(&mut app.dark_mode, "Dark mode");

        if ui.button("Reset View").clicked() {
            app.reset_view();
        }

        if let Some(readout) = &app.cursor_readout {
            let label = format!(
                "{} · x = {:.3}, y = {:.3}",
                readout.label, readout.x, readout.y
            );
            ui.label(RichText::new(label).color(ACCENT_COLOR));
        }
    });

    if let Some(error) = &app.last_error {
        let text = format!("⚠ {error}");
        let error_color = Color32::from_rgb(255, 120, 120);
        ui.colored_label(error_color, text);
    }
}

pub fn sidebar(app: &mut App, ui: &mut egui::Ui) {
    let mut remove_index: Option<usize> = None;
    let total = app.curves.len();

    for idx in 0..total {
        let mut mark_active = false;
        let mut consumed_focus = false;
        let mut request_focus_expr = app.pending_focus() == Some(idx);
        let is_active = app.active_curve == Some(idx);

        {
            let curve = &mut app.curves[idx];
            let frame_fill = if is_active {
                ACCENT_COLOR.linear_multiply(0.18)
            } else {
                Color32::from_black_alpha(16)
            };
            let preview_label = curve.formatted_label(idx);

            egui::Frame::none()
                .fill(frame_fill)
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            let name_response = ui.add(
                                egui::TextEdit::singleline(&mut curve.name)
                                    .hint_text("Name (e.g. f(x))")
                                    .desired_width(140.0),
                            );
                            if name_response.changed() || name_response.has_focus() {
                                mark_active = true;
                            }
                            ui.add_space(8.0);
                            if ui.checkbox(&mut curve.visible, "Visible").changed() {
                                mark_active = true;
                            }
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("Remove").clicked() {
                                        remove_index = Some(idx);
                                    }
                                },
                            );
                        });

                        ui.label(RichText::new(preview_label).small().color(ACCENT_COLOR));

                        ui.separator();

                        ui.label("Function");
                        let expr_response = ui.add(
                            egui::TextEdit::singleline(&mut curve.expr)
                                .hint_text("e.g. sin(x)")
                                .desired_width(f32::INFINITY),
                        );
                        if expr_response.changed() || expr_response.has_focus() {
                            mark_active = true;
                        }

                        ui.horizontal(|ui| {
                            ui.label("Color");
                            if ui.color_edit_button_srgba(&mut curve.color).changed() {
                                mark_active = true;
                            }
                            ui.menu_button("Insert notation", |menu| {
                                for (symbol, snippet, description) in NOTATIONS {
                                    let button_label = format!("{}  {}", symbol, description);
                                    if menu.button(button_label).clicked() {
                                        curve.expr.push_str(snippet);
                                        mark_active = true;
                                        request_focus_expr = true;
                                        menu.close_menu();
                                    }
                                }
                            });
                        });

                        if request_focus_expr {
                            expr_response.request_focus();
                            consumed_focus = true;
                            request_focus_expr = false;
                        }
                    });
                });
        }

        if mark_active {
            app.active_curve = Some(idx);
        }

        if consumed_focus {
            app.clear_pending_focus();
        }

        ui.add_space(10.0);
    }

    if let Some(idx) = remove_index {
        app.remove_curve(idx);
    }

    if ui.button("Add function").clicked() {
        app.add_curve("cos(x)");
    }

    if let Some(idx) = app.pending_focus() {
        if idx >= app.curves.len() {
            app.clear_pending_focus();
        }
    }
}

pub fn plot_panel(app: &mut App, ui: &mut egui::Ui, compiled: &[PreparedCurve]) {
    let reset_view = app.consume_reset_request();

    let mut plot = Plot::new("main_plot")
        .legend(Legend::default())
        .allow_zoom(true)
        .allow_drag(true)
        .show_grid(true)
        .include_y(0.0)
        .include_x(0.0)
        .data_aspect(1.0)
        .auto_bounds(Vec2b::new(false, true));

    if reset_view {
        plot = plot.reset();
    }

    let (target_min, target_max) = app.sampling_bounds();

    let response = plot.show(ui, |plot_ui| {
        let mut bounds = plot_ui.plot_bounds();

        if reset_view {
            let mut min = bounds.min();
            let mut max = bounds.max();
            if !min[1].is_finite() || !max[1].is_finite() || min[1] >= max[1] {
                min[1] = -1.0;
                max[1] = 1.0;
            }
            min[0] = target_min;
            max[0] = target_max;
            let target_bounds = PlotBounds::from_min_max(min, max);
            plot_ui.set_auto_bounds(Vec2b::new(false, true));
            plot_ui.set_plot_bounds(target_bounds);
        }

        bounds = plot_ui.plot_bounds();
        let mut sample_min = target_min;
        let mut sample_max = target_max;

        if bounds.is_valid_x() {
            let x_range = bounds.range_x();
            let view_start = *x_range.start();
            let view_end = *x_range.end();

            if view_start.is_finite() && view_end.is_finite() && view_start < view_end {
                sample_min = view_start;
                sample_max = view_end;
            }
        }

        if !sample_min.is_finite() || !sample_max.is_finite() || !(sample_min < sample_max) {
            sample_min = target_min;
            sample_max = target_max;
        }

        for curve in compiled {
            let points =
                crate::plot::sample_uniform(&curve.func, sample_min, sample_max, app.samples);
            let line = Line::new(points)
                .name(curve.label.clone())
                .color(curve.color)
                .width(2.0);
            plot_ui.line(line);
        }

        update_cursor_readout(app, plot_ui, compiled);
    });

    if response.response.hovered() {
        ui.ctx().request_repaint();
    }
}

fn update_cursor_readout(app: &mut App, plot_ui: &mut PlotUi, curves: &[PreparedCurve]) {
    let pointer = plot_ui.pointer_coordinate();
    app.cursor_readout = None;

    if let Some(pointer) = pointer {
        if !pointer.x.is_finite() || !pointer.y.is_finite() {
            return;
        }

        let bounds = plot_ui.plot_bounds();
        let x_range = bounds.range_x();
        let y_range = bounds.range_y();

        if !x_range.contains(&pointer.x) {
            return;
        }

        let mut best: Option<(usize, f64, Color32, String, bool)> = None;
        let mut best_distance = f64::INFINITY;

        for curve in curves {
            let y = (curve.func)(pointer.x);
            if !y.is_finite() {
                continue;
            }
            let distance = (y - pointer.y).abs();
            let draw_marker = y_range.contains(&y);
            if distance < best_distance {
                best_distance = distance;
                best = Some((
                    curve.index,
                    y,
                    curve.color,
                    curve.label.clone(),
                    draw_marker,
                ));
            }
        }

        if let Some((_, y, color, label, draw_marker)) = best {
            let readout = CursorReadout {
                label: label.clone(),
                x: pointer.x,
                y,
            };

            if draw_marker {
                let marker = Points::new(PlotPoints::from_iter([[pointer.x, y]].into_iter()))
                    .radius(4.0)
                    .color(color)
                    .name("");
                plot_ui.points(marker);

                let text = format!("{}\nx = {:.3}\ny = {:.3}", label, pointer.x, y);
                let annotation = Text::new(PlotPoint::new(pointer.x, y), text).color(color);
                plot_ui.text(annotation);
            }

            app.cursor_readout = Some(readout);
        }
    }
}
