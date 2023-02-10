pub trait Halve: Sized {
    fn halve(self) -> (Self, Self);
}

impl<T: geo::CoordFloat> Halve for geo::Rect<T> {
    fn halve(self) -> (geo::Rect<T>, geo::Rect<T>) {
        if self.width() > self.height() {
            let mid = self.min().x + self.width() / (T::one() + T::one());
            (
                geo::Rect::new(
                    geo::coord! { x: self.min().x, y: self.min().y },
                    geo::coord! { x: mid, y: self.max().y },
                ),
                geo::Rect::new(
                    geo::coord! { x: mid, y: self.min().y },
                    geo::coord! { x: self.max().x, y: self.max().y },
                ),
            )
        } else {
            let mid = self.min().y + self.height() / (T::one() + T::one());
            (
                geo::Rect::new(
                    geo::coord! { x: self.min().x, y: self.min().y },
                    geo::coord! { x: self.max().x, y: mid },
                ),
                geo::Rect::new(
                    geo::coord! { x: self.min().x, y: mid },
                    geo::coord! { x: self.max().x, y: self.max().y },
                ),
            )
        }
    }
}
