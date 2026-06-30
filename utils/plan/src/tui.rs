#![allow(dead_code)]

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortField {
    Type,
    Priority,
    Area,
    Slug,
}

impl SortField {
    fn all() -> [SortField; 4] {
        [
            SortField::Type,
            SortField::Priority,
            SortField::Area,
            SortField::Slug,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            SortField::Type => "Type",
            SortField::Priority => "Priority",
            SortField::Area => "Area",
            SortField::Slug => "Slug",
        }
    }

    fn as_sort_key(&self) -> &'static str {
        match self {
            SortField::Type => "type",
            SortField::Priority => "priority",
            SortField::Area => "area",
            SortField::Slug => "slug",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn symbol(&self) -> &'static str {
        match self {
            SortDirection::Ascending => "▲",
            SortDirection::Descending => "▼",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SortState {
    pub primary: Option<(SortField, SortDirection)>,
    pub secondary: Option<(SortField, SortDirection)>,
}

impl SortState {
    pub fn new() -> Self {
        SortState {
            primary: None,
            secondary: None,
        }
    }

    /// Toggle the given sort field according to the state machine:
    ///
    /// | Current | Press | Result |
    /// |---|---|---|
    /// | Inactive | Space | Becomes primary ▲, old primary → secondary |
    /// | Primary ▲ | Space | Primary ▼ |
    /// | Primary ▼ | Space | Remove. Secondary (if any) → primary |
    /// | Secondary | Space | Swap with primary |
    pub fn toggle(&mut self, field: SortField) {
        if let Some((pf, pd)) = self.primary
            && field == pf
        {
            match pd {
                SortDirection::Ascending => {
                    self.primary = Some((field, SortDirection::Descending));
                }
                SortDirection::Descending => {
                    self.primary = self.secondary.take();
                }
            }
            return;
        }

        if let Some((sf, _sd)) = self.secondary
            && field == sf
        {
            let old_primary = self.primary.take();
            self.primary = self.secondary.take();
            self.secondary = old_primary;
            return;
        }

        let old_primary = self.primary.take();
        self.primary = Some((field, SortDirection::Ascending));
        self.secondary = old_primary;
    }

    /// Returns sort keys with direction for use with plan::cmp_by_key.
    /// The first entry is primary sort, second (if any) is secondary.
    pub fn sort_keys(&self) -> Vec<(&'static str, SortDirection)> {
        let mut keys = Vec::new();
        if let Some((f, d)) = self.primary {
            keys.push((f.as_sort_key(), d));
        }
        if let Some((f, d)) = self.secondary {
            keys.push((f.as_sort_key(), d));
        }
        keys
    }
}

pub fn run_tui(
    root: &Path,
    status_filter: Option<&str>,
    type_filter: Option<&str>,
    priority_filter: Option<&str>,
    area_filter: Option<&str>,
    search_query: Option<&str>,
) {
    let tasks_dir = root.join(".plan").join("tasks");
    let entries = crate::plan::read_task_files(&tasks_dir).unwrap_or_default();
    let tasks = crate::plan::load_and_filter_tasks(
        &entries,
        status_filter,
        type_filter,
        priority_filter,
        area_filter,
        search_query,
    );

    if tasks.is_empty() {
        eprintln!("(no tasks match filters)");
        return;
    }

    eprintln!(
        "plan tui: {} tasks loaded (TUI not yet implemented)",
        tasks.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_state_starts_empty() {
        let s = SortState::new();
        assert!(s.primary.is_none());
        assert!(s.secondary.is_none());
    }

    #[test]
    fn toggle_inactive_becomes_primary() {
        let mut s = SortState::new();
        s.toggle(SortField::Priority);
        assert_eq!(
            s.primary,
            Some((SortField::Priority, SortDirection::Ascending))
        );
        assert!(s.secondary.is_none());
    }

    #[test]
    fn toggle_inactive_pushes_old_primary_to_secondary() {
        let mut s = SortState::new();
        s.toggle(SortField::Priority);
        s.toggle(SortField::Type);
        assert_eq!(s.primary, Some((SortField::Type, SortDirection::Ascending)));
        assert_eq!(
            s.secondary,
            Some((SortField::Priority, SortDirection::Ascending))
        );
    }

    #[test]
    fn toggle_primary_ascending_reverses_to_descending() {
        let mut s = SortState::new();
        s.toggle(SortField::Priority);
        s.toggle(SortField::Priority);
        assert_eq!(
            s.primary,
            Some((SortField::Priority, SortDirection::Descending))
        );
    }

    #[test]
    fn toggle_primary_descending_removes_and_promotes_secondary() {
        let mut s = SortState::new();
        s.toggle(SortField::Priority); // primary = (P, ▲)
        s.toggle(SortField::Type); // primary = (T, ▲), secondary = (P, ▲)
        s.toggle(SortField::Priority); // P is secondary → swap: primary = (P, ▲), secondary = (T, ▲)
        s.toggle(SortField::Priority); // P is primary ▲ → ▼: primary = (P, ▼), secondary = (T, ▲)
        assert_eq!(
            s.primary,
            Some((SortField::Priority, SortDirection::Descending))
        );
        assert_eq!(
            s.secondary,
            Some((SortField::Type, SortDirection::Ascending))
        );
        // Toggle Priority: ▼ → remove, promote Type to primary
        s.toggle(SortField::Priority);
        assert_eq!(s.primary, Some((SortField::Type, SortDirection::Ascending)));
        assert!(s.secondary.is_none());
    }

    #[test]
    fn toggle_secondary_swaps_with_primary() {
        let mut s = SortState::new();
        s.toggle(SortField::Priority); // primary ▲
        s.toggle(SortField::Type); // Type becomes primary ▲, Priority → secondary ▲
        // s.primary = Type, s.secondary = Priority
        s.toggle(SortField::Priority); // secondary → swap: primary = Priority, secondary = Type
        assert_eq!(
            s.primary,
            Some((SortField::Priority, SortDirection::Ascending))
        );
        assert_eq!(
            s.secondary,
            Some((SortField::Type, SortDirection::Ascending))
        );
    }

    #[test]
    fn sort_keys_empty_when_no_keys_active() {
        let s = SortState::new();
        assert!(s.sort_keys().is_empty());
    }

    #[test]
    fn sort_keys_returns_primary_then_secondary() {
        let mut s = SortState::new();
        s.toggle(SortField::Priority); // primary = (P, ▲)
        s.toggle(SortField::Type); // primary = (T, ▲), secondary = (P, ▲)
        let keys = s.sort_keys();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0], ("type", SortDirection::Ascending));
        assert_eq!(keys[1], ("priority", SortDirection::Ascending));
    }

    #[test]
    fn toggle_different_fields_accumulate() {
        let mut s = SortState::new();
        s.toggle(SortField::Type);
        s.toggle(SortField::Priority);
        s.toggle(SortField::Area);
        // Area → primary, Type → secondary (Priority was pushed to nowhere)
        // Wait: let me trace correctly:
        // 1. toggle(Type): primary = Type, secondary = None
        // 2. toggle(Priority): primary = Priority, secondary = Type
        // 3. toggle(Area): primary = Area, secondary = Priority
        assert_eq!(s.primary, Some((SortField::Area, SortDirection::Ascending)));
        assert_eq!(
            s.secondary,
            Some((SortField::Priority, SortDirection::Ascending))
        );
    }

    #[test]
    fn sort_field_as_sort_key_maps_correctly() {
        assert_eq!(SortField::Type.as_sort_key(), "type");
        assert_eq!(SortField::Priority.as_sort_key(), "priority");
        assert_eq!(SortField::Area.as_sort_key(), "area");
        assert_eq!(SortField::Slug.as_sort_key(), "slug");
    }

    #[test]
    fn sort_field_label_is_readable() {
        assert_eq!(SortField::Type.label(), "Type");
        assert_eq!(SortField::Priority.label(), "Priority");
        assert_eq!(SortField::Area.label(), "Area");
        assert_eq!(SortField::Slug.label(), "Slug");
    }

    #[test]
    fn sort_direction_symbol_roundtrips() {
        assert_eq!(SortDirection::Ascending.symbol(), "▲");
        assert_eq!(SortDirection::Descending.symbol(), "▼");
    }
}
