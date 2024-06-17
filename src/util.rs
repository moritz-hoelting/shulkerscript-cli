use std::collections::HashMap;

use camino::Utf8PathBuf;

use inquire::{autocompletion::Replacement, Autocomplete};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PathAutocomplete {
    parent: String,
    current: String,
    outputs: Vec<String>,

    cache: HashMap<String, Vec<String>>,
}

impl PathAutocomplete {
    pub fn new() -> Self {
        Self::default()
    }

    fn split_input(input: &str) -> (&str, &str) {
        let (parent, current) = if input.ends_with('/') {
            (input.trim_end_matches('/'), "")
        } else if let Some((parent, current)) = input.rsplit_once('/') {
            if parent.is_empty() {
                ("/", current)
            } else {
                (parent, current)
            }
        } else {
            ("", input)
        };
        let parent = if parent.is_empty() { "." } else { parent };

        (parent, current)
    }

    fn get_cached(&mut self, parent: &str) -> Result<&[String], &'static str> {
        if !self.cache.contains_key(parent) {
            tracing::trace!("Cache miss for \"{}\", reading dir", parent);

            let parent_path = Utf8PathBuf::from(parent);
            if !parent_path.exists() || !parent_path.is_dir() {
                return Err("Path does not exist");
            }

            let entries = parent_path
                .read_dir_utf8()
                .map_err(|_| "Could not read dir")?
                .filter_map(|entry| {
                    entry.ok().map(|entry| {
                        entry.file_name().to_string() + if entry.path().is_dir() { "/" } else { "" }
                    })
                })
                .collect::<Vec<_>>();

            self.cache.insert(parent.to_string(), entries);
        }

        Ok(self
            .cache
            .get(parent)
            .expect("Previous caching did not work"))
    }

    fn update_input(&mut self, input: &str) -> Result<(), inquire::CustomUserError> {
        let (parent, current) = Self::split_input(input);

        if self.parent == parent && self.current == current {
            Ok(())
        } else {
            self.parent = parent.to_string();
            self.current = current.to_string();

            self.outputs = self
                .get_cached(parent)?
                .iter()
                .filter(|entry| entry.starts_with(current))
                .cloned()
                .collect::<Vec<_>>();

            Ok(())
        }
    }
}

impl Autocomplete for PathAutocomplete {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        self.update_input(input)?;

        Ok(self.outputs.clone())
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        let (parent, current) = Self::split_input(input);

        if let Some(highlighted) = highlighted_suggestion {
            let completion = format!("{parent}/{highlighted}");
            self.update_input(&completion)?;
            Ok(Replacement::Some(completion))
        } else if let Some(first) = self
            .get_cached(parent)?
            .iter()
            .find(|entry| entry.starts_with(current))
        {
            let completion = format!("{parent}/{first}");
            self.update_input(&completion)?;
            Ok(Replacement::Some(completion))
        } else {
            Ok(Replacement::None)
        }
    }
}
