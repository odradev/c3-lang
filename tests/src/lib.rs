use c3_lang_macro::c3_lang;

c3_lang!{
    #[derive(Default)]
    pub struct A {}

    impl A {
        pub fn get(&self) -> u32 {
            10
        }
    }

    #[derive(Default)]
    pub struct B {}

    impl B {
        pub const PARENTS: &'static [ClassName; 1] = &[ClassName::A];

        pub fn get(&self) -> u32 {
            self.super_get() + 1
        }
    }

    #[derive(Default)]
    pub struct C {}

    impl C {
        pub const PARENTS: &'static [ClassName; 1] = &[ClassName::B];

        pub fn get(&self) -> u32 {
            self.super_get() + 1
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{A, B, C};

    #[test]
    fn test_inheritance() {
        let a = A::default();
        assert_eq!(a.get(), 10);
        
        let b = B::default();
        assert_eq!(b.get(), 11);

        let c = C::default();
        assert_eq!(c.get(), 12);

    }
}