#[derive(Copy, Clone)]
pub struct OptionalStruct<T> {
    pub is_some: bool,
    pub value: T,
}

impl <T> OptionalStruct<T> {
    pub fn is_some(&self) -> bool {
        self.is_some
    }

    pub fn is_none(&self) -> bool {
        !self.is_some
    }

    pub fn unwrap(&mut self) -> &mut T {
        if self.is_some {
            self.steal()
        } else {
            panic!("called unwrap on None")
        }
    }

    pub fn steal(&mut self) -> &mut T {
        &mut self.value
    }
}
