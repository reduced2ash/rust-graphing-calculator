use crate::math::CompiledFn;
use egui_plot::PlotPoints;

pub const MIN_SAMPLES: usize = 64;
pub const MAX_SAMPLES: usize = 20_000;

/// Clamp incoming sample counts to a reasonable range.
pub fn clamp_samples(samples: usize) -> usize {
    samples.clamp(MIN_SAMPLES, MAX_SAMPLES)
}

/// Produce regularly sampled points for a compiled function.
pub fn sample_uniform(func: &CompiledFn, x_min: f64, x_max: f64, samples: usize) -> PlotPoints {
    let count = clamp_samples(samples);
    if (x_max - x_min).abs() < f64::EPSILON {
        let y = func(x_min);
        return PlotPoints::from_iter([[x_min, y]].into_iter());
    }

    let steps = (count - 1).max(1) as f64;
    let delta = (x_max - x_min) / steps;
    let mut points = Vec::with_capacity(count);

    for i in 0..count {
        let x = x_min + delta * i as f64;
        let y = func(x);
        if y.is_finite() {
            points.push([x, y]);
        } else {
            // Use NaN to mark discontinuities; egui_plot will break the line.
            points.push([x, f64::NAN]);
        }
    }

    PlotPoints::from_iter(points.into_iter())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_expected_number_of_samples() {
        let func: CompiledFn = std::sync::Arc::new(|x| x);
        let points = sample_uniform(&func, -1.0, 1.0, 128);
        assert_eq!(points.points().len(), clamp_samples(128));
    }
}
