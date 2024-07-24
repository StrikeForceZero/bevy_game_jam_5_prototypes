use bevy::color::LinearRgba;
use bevy::prelude::Color;

pub trait ColorExt {
    fn avg(&self) -> Color;
}

impl ColorExt for [Color] {
    fn avg(&self) -> Color {
        (&self).avg()
    }
}

impl ColorExt for &[Color] {
    fn avg(&self) -> Color {
        let total = self
            .iter()
            .fold(LinearRgba::NONE, |total, &next| total + next.to_linear());
        let mut avg = total / self.len() as f32;
        avg.red = avg.red.clamp(0.0, 1.0);
        avg.blue = avg.blue.clamp(0.0, 1.0);
        avg.green = avg.green.clamp(0.0, 1.0);
        avg.alpha = avg.alpha.clamp(0.0, 1.0);
        Color::LinearRgba(avg)
    }
}
