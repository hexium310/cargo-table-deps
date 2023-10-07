pub(crate) trait Document {}
pub(crate) trait Item {}

pub(crate) trait Table {
    type Item;

    fn insert(&mut self, key: &str, item: Self::Item) -> Option<Self::Item>;
    fn sort_values(&mut self);
    fn set_implicit(&mut self, implicit: bool);
}

pub(crate) trait CfgGetter {
    fn get_cfgs(&self) -> Vec<String>;
}

pub(crate) trait DependenciesGetter {
    type Item: Item;
    type Table: Table<Item = Self::Item> + ExcludeTable<Item = Self::Item>;

    // TODO: Use enum
    fn get_dependencies(&mut self, key: &str) -> Option<&mut Self::Table>;
    fn get_target_dependencies(&mut self, k1: &str, k2: &str) -> Option<&mut Self::Table>;
}

pub(crate) trait ExcludeTable {
    type Item;

    fn exclude_table(&self) -> Vec<(String, Self::Item)>;
}

impl Document for toml_edit::Document {}
impl Item for toml_edit::Item {}

impl Table for toml_edit::Table {
    type Item = toml_edit::Item;

    fn insert(&mut self, key: &str, item: Self::Item) -> Option<Self::Item> {
        self.insert(key, item)
    }

    fn sort_values(&mut self) {
        self.sort_values()
    }

    fn set_implicit(&mut self, implicit: bool) {
        self.set_implicit(implicit)
    }
}

impl CfgGetter for toml_edit::Document {
    fn get_cfgs(&self) -> Vec<String> {
        self.as_table()
            .get("target")
            .and_then(|item| item.as_table())
            .map(|table| table.iter().map(|(key, _)| key.to_string()).collect::<Vec<_>>())
            .unwrap_or_default()
    }
}

impl DependenciesGetter for toml_edit::Document {
    type Item = toml_edit::Item;
    type Table = toml_edit::Table;

    fn get_dependencies(&mut self, key: &str) -> Option<&mut Self::Table> {
        self.get_mut(key).and_then(|item| item.as_table_mut())
    }

    fn get_target_dependencies(&mut self, k1: &str, k2: &str) -> Option<&mut Self::Table> {
        self.get_mut("target")
            .and_then(|item| item.get_mut(k1))
            .and_then(|item| item.get_mut(k2))
            .and_then(|item| item.as_table_mut())
    }
}

impl ExcludeTable for toml_edit::Table {
    type Item = toml_edit::Item;

    fn exclude_table(&self) -> Vec<(String, Self::Item)> {
        self.iter()
            .filter_map(|(key, item)| {
                let Self::Item::Value(value) = item else {
                    return None;
                };

                match value {
                    toml_edit::Value::String(version) => {
                        let mut table = toml_edit::Table::new();
                        table.insert("version", toml_edit::value(version.value()));

                        Some((key.to_string(), toml_edit::Item::Table(table)))
                    },
                    toml_edit::Value::InlineTable(inline_table) => Some((
                        key.to_string(),
                        toml_edit::Item::Table(inline_table.clone().into_table()),
                    )),
                    _ => None,
                }
            })
            .collect()
    }
}
