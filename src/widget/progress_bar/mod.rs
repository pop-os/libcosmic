pub mod circular;
pub mod linear;
pub mod style;

pub fn circular_progress<Message>() -> circular::Circular<crate::Theme> {
    circular::Circular::new()
}

pub fn linear_progress<Message>() -> linear::Linear<crate::Theme> {
    linear::Linear::new()
}
