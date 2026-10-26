//! Registry consistency tests.

use kaffein_i18n::Localizer;

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
    }
}

#[test]
fn every_command_title_and_category_resolves_in_every_locale() {
    let registry = CommandRegistry::new();
    let mut localizer = Localizer::bundled();
    for locale in localizer.available_locales() {
        localizer.set_locale(locale.code()).expect("bundled locale");
        for command in registry.commands() {
            assert!(
                !command.title(&localizer).is_empty(),
                "missing title for '{}' in '{}'",
                command.id,
                locale.code()
            );
            assert!(
                !command.category(&localizer).is_empty(),
                "missing category for '{}' in '{}'",
                command.id,
                locale.code()
            );
        }
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
