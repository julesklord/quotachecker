## 2024-05-23 - Replaced Mock Data with Empty States
**Learning:** Hardcoded mock data can be confusing and alarming in quota/usage tracking tools. Users might interpret mock data as rogue usage or bugs. Displaying fake sessions in the recent sessions tab caused potential user anxiety.
**Action:** Always implement a dedicated "empty state" component with helpful copy (e.g. "No recent sessions found") when data sets are empty, rather than inserting illustrative dummy data.

## 2024-05-24 - Improve keyboard navigation discoverability and accurate keybinds
**Learning:** Terminal applications with hidden keybinds lead to user confusion. Incorrect keybind hints lead to frustration when standard controls (like left/right arrows) map to unexpected actions (global tab switching instead of cycling local values). Context-aware hints (like hiding an edit hint for an uninstalled agent) improve UX by preventing users from attempting invalid actions.
**Action:** Always expose available hotkeys in titles (like tab numbers). Double-check input logic vs instructional text, and dynamically hide hints for unavailable actions.

## 2024-05-25 - Prevent overriding global keybinds with contextual ones
**Learning:** Terminal applications can easily break the user's mental model if contextual keybinds (like changing a setting's value with left/right arrows) accidentally override a global navigation pattern (like changing tabs with left/right arrows), causing them to get "stuck" or perform unintended actions.
**Action:** Do not override global keybinds with contextual/local inputs unless you've implemented an explicit modal, form, or "edit mode" to clearly switch input scopes.

## 2025-07-02 - Hide unavailable context keybinds
**Learning:** In a TUI, context keybinds that show up for uninstalled agents confuse the user.
**Action:** Always dynamically hide keybind hints that prompt users for unavailable actions (like editing quota for uninstalled agent).

## 2024-05-19 - Improved keybind discoverability
**Learning:** Hardcoding generic static hints for a TUI's footer can lead to confusing scenarios when the available actions are highly context-dependent (like navigating different settings). Arrow keys are common TUI patterns, but without explicit string hints (e.g., `Tab/←→`), users might not discover them intuitively.
**Action:** Always conditionally render keybind hints in a TUI to accurately reflect the true available actions for the current selected state/index, and use explicit combined string formats to highlight alternative navigation methods.
## 2026-07-06 - Hide unavailable context keybinds for unlimited agents
**Learning:** In a TUI, context keybinds that show up for uneditable actions confuse the user. I discovered this issue with the 'Unlimited' quota type, where the edit keybind 's' was still visible even though the quota cannot be modified.
**Action:** Always dynamically hide keybind hints that prompt users for unavailable actions (like editing quota for unlimited agents). Ensure backend logic also safely rejects the action with a clear message.
## 2026-07-06 - Missing empty state for tables without rows
**Learning:** Tables in  UI that lack an empty state fallback will render with only headers and a blank body if their backing dataset is empty, causing visual inconsistency and confusion. I encountered this with the model breakdown table in  when no models were available.
**Action:** Always wrap table rendering in an  conditional check. Render a dedicated  with styled copy to clarify the empty state when no data exists.
## 2026-07-06 - Missing empty state for tables without rows
**Learning:** Tables in ratatui UI that lack an empty state fallback will render with only headers and a blank body if their backing dataset is empty, causing visual inconsistency and confusion. I encountered this with the model breakdown table in draw_agents_tab when no models were available.
**Action:** Always wrap table rendering in an .is_empty() conditional check. Render a dedicated Paragraph with styled copy to clarify the empty state when no data exists.

## 2026-07-06 - Modal contextual footer keybinds
**Learning:** If a global TUI footer displays keybinds like `q` for Quit or `Tab` for switching screens, but an interactive modal is currently active on top, users get confused when those global keys are ignored by the modal's event handler.
**Action:** Always conditionally render keybind hints in the global footer based on whether a modal is active. If a modal is shown, only display the modal's specific keybinds (like `Esc` to cancel, `Enter` to save).

## 2026-07-06 - Empty vs invalid form input states
**Learning:** Displaying a bright red error message (e.g., "⚠ Valid number required") when an input field is simply empty (like right after backspacing) is overly aggressive and creates negative user sentiment.
**Action:** Differentiate between empty state and invalid state in forms. Use a neutral, helpful hint (e.g., "ℹ Please enter a numeric limit") when the input is empty, and reserve the red error for actual invalid formats.

## 2026-07-06 - Table column percentages
**Learning:** Ratatui `Table` components with `Constraint::Percentage` columns that sum to less than 100% (e.g., 90%) leave an awkward, unstyled blank space on the right edge of the terminal.
**Action:** When using percentage constraints for a full-width table, ensure the percentages sum to exactly 100%.

## 2024-07-10 - Differentiate between empty state and invalid state in forms
**Learning:** Displaying a bright red error message or border when an input field is simply empty (like right after backspacing or initial opening) is overly aggressive and creates negative user sentiment. It incorrectly flags an incomplete action as an error.
**Action:** Differentiate between empty state and invalid state in forms. Use a neutral, helpful hint (e.g., `COLOR_MUTED` border) when the input is empty, and reserve the red error (`COLOR_DANGER`) for actual invalid formats or limits.
## 2024-05-27 - Prevent global keybind overriding
**Learning:** In terminal applications, users get easily confused when they use an action keybind (like `s` to edit a setting) in a view where that action is completely invalid (like an overview tab), especially if the action is secretly bound globally but only meant to be used contextually.
**Action:** Always ensure action keybindings are strictly scoped to the tabs or views where they are valid, matching the dynamically rendered keybind hints.

## 2026-07-06 - Clarify Selection Indicators
**Learning:** Standard selection indicator symbols (like SYM_ARROW) should be reserved for the selected item, not used to indicate a boolean state like installed.
**Action:** Always map selection symbols strictly to the is_selected state to prevent user confusion.

## 2024-05-28 - Avoid Misleading Navigational Affordances in Settings
**Learning:** Using `<` and `>` around setting values (e.g., `< 1000ms >`) strongly implies left/right arrow key navigation. If left/right arrows are already bound to global navigation (like changing tabs), users will accidentally switch tabs when trying to cycle values, causing frustration.
**Action:** Ensure visual framing of values matches the interaction model. Use neutral brackets like `[ ]` when values are cycled via `Enter` or other keys, reserving `<` and `>` only for actual horizontal navigation.

## 2026-07-18 - Explicit Keybind Alternatives
**Learning:** When alternative keybindings exist for the same action (e.g. 'e' and 'Enter' for opening an editor), omitting one from the footer hints leads to reduced discoverability and potential user frustration.
**Action:** Always combine alternative keybindings in a single hint (e.g. 'Enter/e') if space allows to ensure maximum discoverability.

## 2026-07-20 - Explicit Keybind Alternatives Must Work
**Learning:** Advertising combined alternative keybindings in the UI (e.g., 'Enter/s') creates a strong expectation for the user. If the backend does not actually map the new alternative key to the action, the user perceives the application as broken.
**Action:** When adding explicit keybind alternatives to the UI, always verify that the backend event handlers (e.g., in `main.rs`) correctly process both keycodes for the intended action.
## 2024-07-22 - Explicit Keybind Alternatives for Value Cycling and Quitting
**Learning:** The application had several undocumented keybindings (`Esc` for quitting, `h`/`l` for cycling setting values) that were functional in the backend but invisible to the user. This creates a disconnect where power users or Vim users might try them, but regular users would be unaware they exist, reducing discoverability.
**Action:** When keybindings have alternative options (like `+`/`-` vs `h`/`l` or `q` vs `Esc`), explicitly advertise them in the UI footer using combined string formats like `"Enter/+/-/h/l"` or `"q/Esc"` to ensure all users can discover the full range of supported inputs without guessing.
