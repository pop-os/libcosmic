/// Records if a change has occurred to its inner value
pub struct Track<T> {
    value: T,
    changed: bool,
}

impl<T> Track<T> {
    /// Create a new value where changes are tracked.
    pub const fn new(value: T) -> Self {
        Self {
            value,
            changed: true,
        }
    }

    /// Gets the inner value.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Set a new value, and mark that it has changed.
    pub fn set(&mut self, value: T) {
        self.value = value;
        self.changed = true;
    }

    /// Check if value has changed.
    pub fn changed(&self) -> bool {
        self.changed
    }
}

impl<T> Default for Track<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}
