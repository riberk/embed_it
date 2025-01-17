use std::{borrow::Cow, collections::HashMap};

#[derive(Default)]
pub struct UniqueNames(HashMap<String, usize>);

impl UniqueNames {
    pub fn next<'a>(&mut self, name: &'a str) -> Cow<'a, str> {
        let res = match self.0.get_mut(name) {
            Some(v) => {
                *v += 1;
                Cow::Owned(format!("{}_{}", name, v))
            }
            None => {
                self.0.insert(name.to_owned(), 0);
                Cow::Borrowed(name)
            }
        };
        res
    }
}
