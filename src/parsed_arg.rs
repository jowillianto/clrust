use crate::ArgKey;

#[derive(Debug)]
struct ParamTier {
    value: String,
    params: Vec<(ArgKey, String)>,
}

#[derive(Debug, Default)]
pub struct ParsedArg {
    values: Vec<ParamTier>,
}
impl ParsedArg {
    // Modification Functions
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_positional_argument(&mut self, v: impl Into<String>) -> &mut Self {
        self.values.push(ParamTier {
            value: v.into(),
            params: Vec::new(),
        });
        self
    }
    pub fn add_argument(&mut self, k: impl Into<ArgKey>, v: impl Into<String>) -> &mut Self {
        self.values
            .last_mut()
            .unwrap()
            .params
            .push((k.into(), v.into()));
        self
    }
    pub fn arg(&self) -> &str {
        &self.values.last().unwrap().value
    }
    pub fn param_iter(&self) -> impl Iterator<Item = &(ArgKey, String)> {
        self.values.last().unwrap().params.iter()
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    // Query Function
    pub fn first_of(&self, k: &(impl PartialEq<ArgKey> + ?Sized)) -> Option<&String> {
        match self.param_iter().find(|&(param_key, _)| k == param_key) {
            None => None,
            Some((_, v)) => Some(v),
        }
    }
    pub fn filter<'a>(
        &'a self,
        key: &(impl PartialEq<ArgKey> + ?Sized),
    ) -> impl Iterator<Item = &'a String> {
        self.param_iter()
            .filter(move |&arg| key == &arg.0)
            .map(move |arg| &arg.1)
    }
    pub fn count(&self, key: &(impl PartialEq<ArgKey> + ?Sized)) -> usize {
        self.filter(key).count()
    }
    pub fn contains(&self, key: &(impl PartialEq<ArgKey> + ?Sized)) -> bool {
        self.first_of(key).is_some()
    }
}
