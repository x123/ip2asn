use std::collections::HashMap;

#[derive(Debug, Default)]
pub(crate) struct StringInterner {
    map: HashMap<String, u32>,
    vec: Vec<String>,
}

impl StringInterner {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get_or_intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.map.get(s) {
            return id;
        }
        let id = self.vec.len() as u32;
        self.map.insert(s.to_owned(), id);
        self.vec.push(s.to_owned());
        id
    }

    pub(crate) fn into_vec(self) -> Vec<String> {
        self.vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();

        // Intern a new string, should get ID 0
        let id1 = interner.get_or_intern("Apple Inc.");
        assert_eq!(id1, 0);

        // Intern another new string, should get ID 1
        let id2 = interner.get_or_intern("Google LLC");
        assert_eq!(id2, 1);

        // Intern the first string again, should get the same ID 0
        let id3 = interner.get_or_intern("Apple Inc.");
        assert_eq!(id3, 0);

        // Consume the interner and check the resulting vector
        let strings = interner.into_vec();
        assert_eq!(strings.len(), 2);
        assert_eq!(strings[id1 as usize], "Apple Inc.");
        assert_eq!(strings[id2 as usize], "Google LLC");
    }
}
