pub trait StyleBuilder<T> {
    fn style(self, style: impl Fn(T) -> T) -> Self;
}

