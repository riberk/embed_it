use std::collections::HashMap;

#[derive(Default)]
pub struct UniqueNames(HashMap<String, usize>);

impl UniqueNames {
    pub fn next(&mut self, name: &str) -> Option<usize> {
        match self.0.get_mut(name) {
            Some(v) => {
                *v += 1;
                Some(*v)
            }
            None => {
                self.0.insert(name.to_owned(), 0);
                None
            }
        }
    }
}

#[derive(Default)]
pub struct UniqueIdents {
    module_like: UniqueNames,
    struct_like: UniqueNames,
}

impl UniqueIdents {
    pub fn next_module(&mut self, name: &str) -> Option<usize> {
        self.module_like.next(name)
    }

    pub fn next_struct(&mut self, name: &str) -> Option<usize> {
        self.struct_like.next(name)
    }
}
