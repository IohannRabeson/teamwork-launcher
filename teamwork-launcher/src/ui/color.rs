use iced::Color;

pub fn brighter(color: Color) -> Color {
    brighter_by(color, 0.1)
}

pub fn brighter_by(color: Color, value: f32) -> Color {
    let min = |x: f32, y: f32| {
        if x < y {
            x
        } else {
            y
        }
    };

    Color {
        r: min(1.0, color.r + value),
        g: min(1.0, color.g + value),
        b: min(1.0, color.b + value),
        a: color.a,
    }
}
