# Project Cookbooks

This directory contains project-specific testing guides showing how to use terminal-testlib in different application contexts.

## Available Cookbooks

### [Scarab - File Manager](scarab.md)
Testing patterns for file manager TUIs:
- File list navigation
- Directory tree operations
- Dual-pane layouts
- File operation confirmations

**Best for**: File browsers, file managers, directory navigators

---

### [Scarab-Nav - Navigation Framework](scarab-nav.md)
Navigation and interaction patterns:
- Vim-style keybindings
- Modal dialogs
- Quick action menus
- State management

**Best for**: Vim-like interfaces, modal navigation, keyboard-driven UIs

---

### [Tolaria - MTG Deck Manager](tolaria.md)
Data-intensive application patterns:
- Card database searching
- List filtering and sorting
- Deck validation
- Complex table layouts

**Best for**: Database UIs, data management tools, inventory systems

---

### [Sparky - EV Dashboard](sparky.md)
Real-time dashboard patterns:
- Live data updates
- Gauge and chart widgets
- Alert notifications
- Multi-screen workflows

**Best for**: Dashboards, monitoring tools, real-time applications

---

## Using These Cookbooks

Each cookbook follows the same structure:

1. **Overview** - Project context and testing goals
2. **Setup** - Test harness configuration
3. **Common Patterns** - Reusable testing patterns
4. **Real-World Examples** - Complete test cases
5. **Troubleshooting** - Common issues and solutions

### Getting Started

1. Choose the cookbook closest to your use case
2. Review the setup section
3. Explore common patterns relevant to your needs
4. Adapt examples to your application

### Contributing

To add a new cookbook:

1. Copy an existing cookbook as a template
2. Fill in project-specific patterns
3. Include working code examples
4. Test all examples
5. Submit a PR

## Cross-References

Cookbooks reference common functionality from:
- [Core API Documentation](https://docs.rs/terminal-testlib)
- [Examples Gallery](../../../../examples/)
- [ROADMAP.md](../ROADMAP.md) - Planned features

## Questions?

- For cookbook-specific questions, see the Troubleshooting section in each guide
- For general terminal-testlib questions, see the main [README](../../../../README.md)
- For bugs or feature requests, open an issue on GitHub
