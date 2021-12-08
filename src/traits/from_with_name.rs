pub trait FromWithName<T> {
    fn from_with_name(name: String, from: T) -> Self;
}

pub trait IntoWithName<T> {
    fn into_with_name(self, name: String) -> T;
}

impl<R, T> IntoWithName<T> for R
where
    T: FromWithName<R>,
{
    fn into_with_name(self, name: String) -> T {
        T::from_with_name(name, self)
    }
}
