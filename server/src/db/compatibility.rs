pub trait BooleanLike {
    fn is_true(&self) -> bool;
}

impl BooleanLike for bool {
    fn is_true(&self) -> bool {
        *self
    }
}

impl BooleanLike for i8 {
    fn is_true(&self) -> bool {
        *self != 0
    }
}
