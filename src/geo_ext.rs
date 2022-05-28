pub trait Cast<To: geo::CoordNum> {
    type Output;

    fn cast(self) -> Option<Self::Output>;
}

impl<From: geo::CoordNum, To: geo::CoordNum> Cast<To> for geo::Rect<From> {
    type Output = geo::Rect<To>;

    fn cast(self) -> Option<Self::Output> {
        Some(geo::Rect::new(
            geo::coord! {
                x: To::from(self.min().x)?,
                y: To::from(self.min().y)?
            },
            geo::coord! {
                x: To::from(self.max().x)?,
                y: To::from(self.max().y)?
            },
        ))
    }
}
