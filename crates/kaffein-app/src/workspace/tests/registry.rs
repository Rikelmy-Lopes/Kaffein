//! Registry consistency tests.

use crate::command::Menu;
use crate::registry::CommandRegistry;

#[test]
fn registry_ids_are_unique_and_namespaced() {
    let registry = CommandRegistry::new();
    let mut seen = std::collections::HashSet::new();
    for command in registry.commands() {
        assert!(
            command.id.contains('.'),
            "command id '{}' is not namespaced",
            command.id
        );
        assert!(
            seen.insert(command.id),
            "duplicate command id '{}'",
            command.id
        );
        assert!(!command.title.is_empty());
    }
}

#[test]
fn every_menu_and_toolbar_entry_resolves_to_a_command() {
    let registry = CommandRegistry::new();
    for menu in Menu::ALL {
        for command in registry.menu_commands(menu) {
            assert!(registry.get(command.id).is_some());
        }
    }
    for command in registry.toolbar_commands() {
        assert!(registry.get(command.id).is_some());
    }
}
