#![doc = include_str!("../README.md")]
use serde::{ser::SerializeMap, Serialize};

#[derive(Debug, Serialize)]
#[serde(tag = "entry_difference", rename_all = "snake_case")]
pub enum EntryDifference {
    /// An entry from `target` that `source` is missing
    Missing { value: serde_json::Value },
    /// An entry that `source` has, and `target` doesn't
    Extra { value: serde_json::Value },
    /// The entry exists in both JSONs, but the values are different
    Value { value_diff: Difference },
}

#[derive(Debug)]
pub struct DumbMap<K: Serialize, V: Serialize>(pub Vec<(K, V)>);

impl<K: Serialize, V: Serialize> Serialize for DumbMap<K, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in &self.0 {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "array_difference", rename_all = "snake_case")]
pub enum ArrayDifference {
    /// `source` and `target` are the same length, but some values of the same indices are different
    PairsOnly {
        /// differing pairs that appear in the overlapping indices of `source` and `target`
        different_pairs: DumbMap<usize, Difference>,
    },
    /// `source` is shorter than `target`
    Shorter {
        /// differing pairs that appear in the overlapping indices of `source` and `target`
        different_pairs: Option<DumbMap<usize, Difference>>,
        /// elements missing in `source` that appear in `target`
        missing_elements: Vec<serde_json::Value>,
    },
    /// `source` is longer than `target`
    Longer {
        /// differing pairs that appear in the overlapping indices of `source` and `target`
        different_pairs: Option<DumbMap<usize, Difference>>,
        /// The amount of extra elements `source` has that `target` does not
        extra_length: usize,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Null,
    Array,
    Bool,
    Object,
    String,
    Number,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ScalarDifference {
    Bool {
        source: bool,
        target: bool,
    },
    String {
        source: String,
        target: String,
    },
    Number {
        source: serde_json::Number,
        target: serde_json::Number,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "difference_of", rename_all = "snake_case")]
pub enum Difference {
    Scalar(ScalarDifference),
    Type {
        source_type: Type,
        source_value: serde_json::Value,
        target_type: Type,
        target_value: serde_json::Value,
    },
    Array(ArrayDifference),
    Object {
        different_entries: DumbMap<String, EntryDifference>,
    },
}

pub struct Diff {
    filters: Vec<Filter>,
    curr_path: Vec<ElementPath>,
}

impl Diff {
    pub fn new(filters: Vec<Filter>) -> Self {
        Self {
            filters,
            curr_path: Vec::new(),
        }
    }

    #[must_use]
    fn arrays(
        &self,
        source: Vec<serde_json::Value>,
        target: Vec<serde_json::Value>,
        curr_path: Vec<ElementPath>,
    ) -> Option<ArrayDifference> {
        let different_pairs = self.compare_array_elements(&source, &target, &curr_path);
        let different_pairs = if different_pairs.is_empty() {
            None
        } else {
            Some(DumbMap(different_pairs))
        };

        match (source.len(), target.len()) {
            (s, t) if s > t => Some(ArrayDifference::Longer {
                different_pairs,
                extra_length: s - t,
            }),
            (s, t) if s < t => Some(ArrayDifference::Shorter {
                different_pairs,
                missing_elements: target.into_iter().skip(s).collect(),
            }),
            _ => different_pairs.map(|pairs| ArrayDifference::PairsOnly { different_pairs: pairs }),
        }
    }

    fn compare_array_elements(
        &self,
        source: &[serde_json::Value],
        target: &[serde_json::Value],
        curr_path: &[ElementPath],
    ) -> Vec<(usize, Difference)> {
        source
            .iter()
            .zip(target.iter())
            .enumerate()
            .filter_map(|(i, (s, t))| {
                let mut path = curr_path.to_vec();
                path.push(ElementPath::ArrayIndex(ArrayIndex::Index(i)));
                self.values(s.clone(), t.clone(), path).map(|diff| (i, diff))
            })
            .collect()
    }

    #[must_use]
    fn objects(
        &self,
        source: serde_json::Map<String, serde_json::Value>,
        mut target: serde_json::Map<String, serde_json::Value>,
        mut curr_path: Vec<ElementPath>,
    ) -> Option<DumbMap<String, EntryDifference>> {
    let mut value_differences = source
        .into_iter()
        .filter_map(|(key, source)| {
            let mut curr_path = curr_path.clone();
            let curr_path: &mut Vec<ElementPath> = curr_path.as_mut();
            curr_path.push(ElementPath::Key(key.clone()));

            let Some(target) = target.remove(&key) else {
                if ignore(&filters, &curr_path) {
                    return None;
                }

                return Some((key, EntryDifference::Extra {
                    value: source
                }));
            };

            self.values(source, target, curr_path.clone()).map(|diff| (key, EntryDifference::Value { value_diff: diff }))
        })
        .collect::<Vec<_>>();

    value_differences.extend(target.into_iter().map(|(missing_key, missing_value)| {
        (
            missing_key,
            EntryDifference::Missing {
                value: missing_value,
            },
        )
    }));

    if value_differences.is_empty() {
        None
    } else {
        Some(DumbMap(value_differences))
    }
}

impl From<serde_json::Value> for Type {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Type::Null,
            serde_json::Value::Bool(_) => Type::Bool,
            serde_json::Value::Number(_) => Type::Number,
            serde_json::Value::String(_) => Type::String,
            serde_json::Value::Array(_) => Type::Array,
            serde_json::Value::Object(_) => Type::Object,
        }
    }
}

    #[must_use]
    pub fn values(&self, source: serde_json::Value, target: serde_json::Value, mut curr_path: Vec<ElementPath>) -> Option<Difference> {
    use serde_json::Value::{Array, Bool, Null, Number, Object, String};

    if ignore(&filters, &curr_path) {
        return None;
    }

    match (source, target) {
        (Null, Null) => None,
        (Bool(source), Bool(target)) => {
            if source == target {
                None
            } else {
                Some(Difference::Scalar(ScalarDifference::Bool {
                    source,
                    target,
                }))
            }
        }
        (Number(source), Number(target)) => {
            if source == target {
                None
            } else {
                Some(Difference::Scalar(ScalarDifference::Number {
                    source,
                    target,
                }))
            }
        }
        (String(source), String(target)) => {
            if source == target {
                None
            } else {
                Some(Difference::Scalar(ScalarDifference::String {
                    source,
                    target,
                }))
            }
        }
        (Array(source), Array(target)) => self.arrays(source, target, curr_path.clone()).map(Difference::Array),
        (Object(source), Object(target)) => self.objects(source, target, curr_path.clone())
            .map(|different_entries| Difference::Object { different_entries }),
        (source, target) => Some(Difference::Type {
            source_type: source.clone().into(),
            source_value: source,
            target_type: target.clone().into(),
            target_value: target,
        }),
    }
}

fn ignore(filters: &[Filter], curr_path: &[ElementPath]) -> bool {
    let elem_path_filters: Vec<_> = filters.iter().filter_map(|x| {
        match x {
            Filter::Ignore(path) => { Some(path) }
        }
    }).collect();

    elem_path_filters.iter().any(|&x| x.eq(curr_path))
}

#[derive(Eq, Clone, Debug)]
pub enum ArrayIndex {
    Index(usize),
    All,
}

impl PartialEq for ArrayIndex {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ArrayIndex::Index(a), ArrayIndex::Index(b)) => a == b,
            (ArrayIndex::All, ArrayIndex::Index(_)) => true,
            (ArrayIndex::Index(_), ArrayIndex::All) => true,
            (ArrayIndex::All, ArrayIndex::All) => true,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ElementPath {
    Key(String),
    ArrayIndex(ArrayIndex),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Filter {
    /// Completely ignore differences at the specified paths
    Ignore(Vec<ElementPath>),
    /// Only compare values at the specified paths
    Only(Vec<ElementPath>),
    /// Ignore array ordering at the specified paths
    IgnoreOrder(Vec<ElementPath>),
}
