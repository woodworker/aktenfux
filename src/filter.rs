use crate::frontmatter::Note;
use yaml_rust2::Yaml;
use std::collections::HashMap;
use crate::yaml_compat::{yaml_to_string, collect_yaml_strings};

pub struct FilterCriteria {
    filters: Vec<(String, String)>,
}

impl FilterCriteria {
    pub fn new(filters: Vec<(String, String)>) -> Self {
        Self { filters }
    }

    pub fn apply_filters<'a>(&self, notes: &'a [Note]) -> Vec<&'a Note> {
        if self.filters.is_empty() {
            return notes.iter().collect();
        }

        notes
            .iter()
            .filter(|note| self.matches_all_filters(note))
            .collect()
    }

    fn matches_all_filters(&self, note: &Note) -> bool {
        self.filters
            .iter()
            .all(|(key, value)| note.matches_filter(key, value))
    }
}

pub fn collect_all_fields(notes: &[Note]) -> Vec<String> {
    let mut all_fields = std::collections::HashSet::new();
    
    for note in notes {
        for key in note.frontmatter.keys() {
            all_fields.insert(key.clone());
        }
    }
    
    let mut fields: Vec<String> = all_fields.into_iter().collect();
    fields.sort();
    fields
}

pub fn collect_field_values(notes: &[Note], field: &str) -> Vec<String> {
    let mut all_values = std::collections::HashSet::new();
    
    for note in notes {
        if let Some(value) = note.get_frontmatter_value(field) {
            let strings = collect_yaml_strings(value);
            for s in strings {
                all_values.insert(s);
            }
        }
    }
    
    let mut values: Vec<String> = all_values.into_iter().collect();
    values.sort();
    values
}

pub fn get_field_statistics(notes: &[Note]) -> HashMap<String, FieldStats> {
    let mut stats = HashMap::new();
    
    for note in notes {
        for (key, value) in &note.frontmatter {
            let field_stats = stats.entry(key.clone()).or_insert_with(FieldStats::new);
            field_stats.increment(value);
        }
    }
    
    stats
}

#[derive(Debug)]
pub struct FieldStats {
    pub total_count: usize,
    pub unique_values: std::collections::HashSet<String>,
    pub value_counts: HashMap<String, usize>,
}

impl FieldStats {
    fn new() -> Self {
        Self {
            total_count: 0,
            unique_values: std::collections::HashSet::new(),
            value_counts: HashMap::new(),
        }
    }
    
    fn increment(&mut self, value: &Yaml) {
        self.total_count += 1;
        
        match value {
            Yaml::String(s) => {
                self.unique_values.insert(s.clone());
                *self.value_counts.entry(s.clone()).or_insert(0) += 1;
            }
            Yaml::Array(arr) => {
                for item in arr {
                    if let Yaml::String(s) = item {
                        self.unique_values.insert(s.clone());
                        *self.value_counts.entry(s.clone()).or_insert(0) += 1;
                    }
                }
            }
            Yaml::Integer(n) => {
                let s = n.to_string();
                self.unique_values.insert(s.clone());
                *self.value_counts.entry(s).or_insert(0) += 1;
            }
            Yaml::Real(f) => {
                let s = f.to_string();
                self.unique_values.insert(s.clone());
                *self.value_counts.entry(s).or_insert(0) += 1;
            }
            Yaml::Boolean(b) => {
                let s = b.to_string();
                self.unique_values.insert(s.clone());
                *self.value_counts.entry(s).or_insert(0) += 1;
            }
            _ => {
                let s = yaml_to_string(value);
                self.unique_values.insert(s.clone());
                *self.value_counts.entry(s).or_insert(0) += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_note(path: &str, frontmatter: HashMap<String, Yaml>) -> Note {
        Note::new(path.to_string(), frontmatter)
    }

    #[test]
    fn test_filter_criteria() {
        let mut fm1 = HashMap::new();
        fm1.insert("tag".to_string(), Yaml::String("work".to_string()));
        fm1.insert("status".to_string(), Yaml::String("active".to_string()));
        
        let mut fm2 = HashMap::new();
        fm2.insert("tag".to_string(), Yaml::String("personal".to_string()));
        fm2.insert("status".to_string(), Yaml::String("active".to_string()));
        
        let notes = vec![
            create_test_note("note1.md", fm1),
            create_test_note("note2.md", fm2),
        ];
        
        let criteria = FilterCriteria::new(vec![("tag".to_string(), "work".to_string())]);
        let filtered = criteria.apply_filters(&notes);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].path, "note1.md");
    }

    #[test]
    fn test_collect_all_fields() {
        let mut fm1 = HashMap::new();
        fm1.insert("title".to_string(), Yaml::String("Note 1".to_string()));
        fm1.insert("tag".to_string(), Yaml::String("work".to_string()));
        
        let mut fm2 = HashMap::new();
        fm2.insert("title".to_string(), Yaml::String("Note 2".to_string()));
        fm2.insert("status".to_string(), Yaml::String("active".to_string()));
        
        let notes = vec![
            create_test_note("note1.md", fm1),
            create_test_note("note2.md", fm2),
        ];
        
        let fields = collect_all_fields(&notes);
        assert_eq!(fields.len(), 3);
        assert!(fields.contains(&"title".to_string()));
        assert!(fields.contains(&"tag".to_string()));
        assert!(fields.contains(&"status".to_string()));
    }
}