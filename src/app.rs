use std::time::Duration;

use crate::math::{self, CompiledFn};
use crate::plot;
use crate::store::{self, StoredCurve, StoredState};
use crate::ui;
use egui::{self, Color32, Context, Frame, Margin, Rounding};

fn fallback_curve_name(index: usize) -> String {
    format!("f{}(x)", index + 1)
}

const DEFAULT_CURVES: &[(&str, Color32)] = &[
    ("sin(x)", Color32::from_rgb(80, 145, 255)),
    ("x^2", Color32::from_rgb(255, 120, 80)),
    ("cos(x)", Color32::from_rgb(120, 220, 160)),
    ("exp(x)", Color32::from_rgb(200, 170, 255)),
    ("ln(x)", Color32::from_rgb(255, 210, 110)),
    ("tan(x)", Color32::from_rgb(250, 150, 220)),
];

const DEFAULT_GRAPH_RANGE: (f64, f64) = (-10.0, 10.0);

#[derive(Debug, Clone)]
pub struct Curve {
    pub name: String,
    pub expr: String,
    pub color: Color32,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct CursorReadout {
    pub label: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone)]
pub struct PreparedCurve {
    pub index: usize,
    pub label: String,
    pub color: Color32,
    pub func: CompiledFn,
}

pub struct App {
    pub curves: Vec<Curve>,
    pub x_min: f64,
    pub x_max: f64,
    pub samples: usize,
    pub last_error: Option<String>,
    pub cursor_readout: Option<CursorReadout>,
    pub dark_mode: bool,
    pub active_curve: Option<usize>,
    pending_focus: Option<usize>,
    reset_requested: bool,
    next_color_index: usize,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        match store::load_session() {
            Ok(Some(saved)) => Self::from_stored(saved),
            Ok(None) | Err(_) => Self::default(),
        }
    }

    fn from_stored(state: StoredState) -> Self {
        let mut curves = state
            .curves
            .into_iter()
            .map(Curve::from_stored)
            .collect::<Vec<_>>();
        if curves.is_empty() {
            curves = Self::default_curves();
        }

        let next_color_index = curves.len();
        let mut app = Self {
            curves,
            x_min: state.x_min,
            x_max: state.x_max,
            samples: state.samples,
            last_error: None,
            cursor_readout: None,
            dark_mode: state.dark_mode,
            active_curve: None,
            pending_focus: None,
            reset_requested: false,
            next_color_index,
        };
        app.ensure_curve_names();
        app.ensure_valid_settings();
        app
    }

    fn default_curves() -> Vec<Curve> {
        DEFAULT_CURVES
            .iter()
            .take(2)
            .enumerate()
            .map(|(idx, (expr, color))| Curve {
                name: fallback_curve_name(idx),
                expr: (*expr).to_owned(),
                color: *color,
                visible: true,
            })
            .collect()
    }

    fn ensure_curve_names(&mut self) {
        for (idx, curve) in self.curves.iter_mut().enumerate() {
            if curve.name.trim().is_empty() {
                curve.name = fallback_curve_name(idx);
            }
        }
    }

    fn next_curve_color(&mut self) -> Color32 {
        let color = DEFAULT_CURVES
            .get(self.next_color_index % DEFAULT_CURVES.len())
            .map(|(_, c)| *c)
            .unwrap_or(Color32::from_rgb(100, 200, 220));
        self.next_color_index += 1;
        color
    }

    fn ensure_valid_settings(&mut self) {
        if !self.x_min.is_finite() && !self.x_max.is_finite() {
            self.x_min = DEFAULT_GRAPH_RANGE.0;
            self.x_max = DEFAULT_GRAPH_RANGE.1;
        } else if !self.x_min.is_finite() {
            self.x_min = self.x_max - (DEFAULT_GRAPH_RANGE.1 - DEFAULT_GRAPH_RANGE.0);
        } else if !self.x_max.is_finite() {
            self.x_max = self.x_min + (DEFAULT_GRAPH_RANGE.1 - DEFAULT_GRAPH_RANGE.0);
        }

        if self.x_min >= self.x_max {
            self.x_max = self.x_min + (DEFAULT_GRAPH_RANGE.1 - DEFAULT_GRAPH_RANGE.0).max(1.0);
        }

        self.samples = plot::clamp_samples(self.samples);
    }

    pub fn pending_focus(&self) -> Option<usize> {
        self.pending_focus
    }

    pub fn clear_pending_focus(&mut self) {
        self.pending_focus = None;
    }

    pub fn consume_reset_request(&mut self) -> bool {
        if self.reset_requested {
            self.reset_requested = false;
            true
        } else {
            false
        }
    }

    pub fn sampling_bounds(&self) -> (f64, f64) {
        let default = DEFAULT_GRAPH_RANGE;
        let default_span = default.1 - default.0;
        let mut min = self.x_min;
        let mut max = self.x_max;

        match (min.is_finite(), max.is_finite()) {
            (true, true) => {}
            (false, true) => {
                min = max - default_span;
            }
            (true, false) => {
                max = min + default_span;
            }
            (false, false) => {
                min = default.0;
                max = default.1;
            }
        }

        if !min.is_finite() || !max.is_finite() {
            min = default.0;
            max = default.1;
        }

        if min >= max {
            max = min + default_span.max(1.0);
        }

        (min, max)
    }

    fn snapshot(&self) -> StoredState {
        StoredState {
            curves: self.curves.iter().map(Curve::to_stored).collect(),
            x_min: self.x_min,
            x_max: self.x_max,
            samples: self.samples,
            dark_mode: self.dark_mode,
        }
    }

    pub fn add_curve(&mut self, expr: &str) {
        let color = self.next_curve_color();
        let index = self.curves.len();
        let name = fallback_curve_name(index);
        self.curves.push(Curve {
            name,
            expr: expr.to_owned(),
            color,
            visible: true,
        });
        self.active_curve = Some(index);
        self.pending_focus = Some(index);
    }

    pub fn remove_curve(&mut self, index: usize) {
        if self.curves.len() > 1 && index < self.curves.len() {
            self.curves.remove(index);
            if let Some(active) = self.active_curve {
                self.active_curve = if active == index {
                    None
                } else if active > index {
                    Some(active - 1)
                } else {
                    Some(active)
                };
            }
            if let Some(pending) = self.pending_focus {
                self.pending_focus = if pending == index {
                    None
                } else if pending > index {
                    Some(pending - 1)
                } else {
                    Some(pending)
                };
            }
            self.ensure_curve_names();
        }
    }

    pub fn reset_view(&mut self) {
        self.x_min = DEFAULT_GRAPH_RANGE.0;
        self.x_max = DEFAULT_GRAPH_RANGE.1;
        self.request_view_refresh();
    }

    pub fn request_view_refresh(&mut self) {
        self.reset_requested = true;
        self.cursor_readout = None;
    }

    fn compile_curves(&mut self) -> Vec<PreparedCurve> {
        let mut compiled = Vec::with_capacity(self.curves.len());
        self.last_error = None;

        for (idx, curve) in self.curves.iter().enumerate() {
            if !curve.visible || curve.expr.trim().is_empty() {
                continue;
            }

            let label = curve.formatted_label(idx);
            match math::compile_expr(&curve.expr) {
                Ok(func) => compiled.push(PreparedCurve {
                    index: idx,
                    label,
                    color: curve.color,
                    func,
                }),
                Err(err) => {
                    if self.last_error.is_none() {
                        self.last_error = Some(format!("{}: {}", label, err));
                    }
                }
            }
        }

        compiled
    }

    fn update_plot(&mut self, ctx: &Context) -> Vec<PreparedCurve> {
        let compiled = self.compile_curves();
        if !compiled.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        compiled
    }
}

impl Default for App {
    fn default() -> Self {
        let curves = Self::default_curves();
        let next_color_index = curves.len();
        let mut app = Self {
            curves,
            x_min: DEFAULT_GRAPH_RANGE.0,
            x_max: DEFAULT_GRAPH_RANGE.1,
            samples: 3_000,
            last_error: None,
            cursor_readout: None,
            dark_mode: true,
            active_curve: None,
            pending_focus: None,
            reset_requested: false,
            next_color_index,
        };
        app.ensure_curve_names();
        app
    }
}

impl Curve {
    pub(crate) fn display_name(&self, index: usize) -> String {
        let trimmed = self.name.trim();
        if trimmed.is_empty() {
            fallback_curve_name(index)
        } else {
            trimmed.to_owned()
        }
    }

    pub(crate) fn formatted_label(&self, index: usize) -> String {
        let name = self.display_name(index);
        let pretty = math::prettify_expression(&self.expr);
        format!("{} = {}", name, pretty)
    }

    fn to_stored(&self) -> StoredCurve {
        let [r, g, b, a] = self.color.to_array();
        StoredCurve {
            name: self.name.clone(),
            expr: self.expr.clone(),
            color: [r, g, b, a],
            visible: self.visible,
        }
    }

    fn from_stored(stored: StoredCurve) -> Self {
        let [r, g, b, a] = stored.color;
        Self {
            name: stored.name,
            expr: stored.expr,
            color: Color32::from_rgba_unmultiplied(r, g, b, a),
            visible: stored.visible,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.ensure_valid_settings();
        ui::install_visuals(ctx, self.dark_mode);

        let compiled = self.update_plot(ctx);

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui::top_bar(self, ui);
        });

        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui::sidebar(self, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let frame = Frame {
                inner_margin: Margin::same(12.0),
                rounding: Rounding::same(16.0),
                ..Default::default()
            };
            frame.show(ui, |ui| {
                ui::plot_panel(self, ui, &compiled);
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(err) = store::save_session(&self.snapshot()) {
            eprintln!("failed to save session: {err}");
        }
    }
}
