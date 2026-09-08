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
## 2024-07-23 - Provide Vim-style navigation alternatives
**Learning:** In TUI applications, providing standard arrow key navigation (`↑↓`) is good, but omitting Vim-style alternatives (`j`/`k`) alienates power users who prefer to keep their hands on the home row.
**Action:** Always map `j` and `k` to down/up actions in list views and explicitly advertise them in UI hints (e.g., `↑↓/k/j`) alongside standard arrow keys to improve accessibility and discoverability.
## 2024-07-25 - Support Shift-Tab for backwards navigation
**Learning:** In TUI applications, users expect standard browser-like bidirectional navigation using Tab and Shift-Tab. Missing Shift-Tab support breaks user flow and decreases accessibility, as users must cycle through all tabs forward to go back one step.
**Action:** Always implement `KeyCode::BackTab` alongside `KeyCode::Tab` for bidirectional list/tab navigation, and ensure UI hints (like `Tab/Shift+Tab`) advertise this capability.
## 2024-07-28 - Inline validation for lower bounds
**Learning:** When users edit numeric limits (like quotas), allowing them to set limits below their current usage without a clear warning can cause accidental immediate lockouts or confusion.
**Action:** Add inline validation warnings when a new proposed limit is below the currently consumed quota amount.

## 2024-07-29 - Provide contextual data in modals to prevent memory reliance
**Learning:** When users edit a limit or quota in a modal, only validating the new input against the current usage (e.g. showing "Limit is below current usage") without actually displaying the current usage amount in the modal itself forces the user to rely on memory. This breaks their flow as they might have to close the modal to check their usage before trying again.
**Action:** Always provide necessary contextual data (like current usage) directly within the modal or form so users can make informed decisions without navigating away.

## 2026-07-30 - Missing empty state for tables without rows
**Learning:** Tables in ratatui UI that lack an empty state fallback will render with only headers and a blank body if their backing dataset is empty, causing visual inconsistency and confusion. I encountered this with the summary_table, budget_table, and config_table when their data vectors were empty.
**Action:** Always wrap table rendering in an `.is_empty()` conditional check. Render a dedicated `Paragraph` with styled copy to clarify the empty state when no data exists.
## 2026-07-31 - Transient status feedback in footer
**Learning:** Missing global success/error toasts for contextual actions (like saving settings or editing quotas) leaves users uncertain if their action succeeded, especially if the logs are only visible on a specific tab. TUIs should repurpose global footers for transient status feedback.
**Action:** Add a log display section to the global footer to show the latest system log message across all tabs.
## 2024-07-31 - Differentiate identical value input in forms
**Learning:** In TUI modals, when a user inputs a value identical to the currently active state (e.g. entering '100' when the limit is already '100') and no specific feedback is provided, users may be confused whether their action succeeded or was ignored.
**Action:** When validating form inputs that match the existing state, display a neutral, informative message (e.g., 'Limit is unchanged') instead of falling back to default behavior or falsely implying a change was made.
## 2024-08-15 - Specific error messages for type-bound overflows
**Learning:** When form inputs are pre-filtered to only accept digits, generic "Valid number required" errors are confusing when users hit a type-bound size limit (e.g., u32 max). They assume their format is wrong rather than the value being too large.
**Action:** In TUI form validations, if an input field is already restricted to digits, use specific error messages (e.g., 'Number too large') for type-bound overflow rather than generic 'Valid number required' errors to reduce user confusion.
## 2023-10-27 - Actionable CTAs in Empty States
**Learning:** Empty states without Call-to-Actions (CTAs) leave users confused in terminal applications where standard navigation paradigms are not obvious. Providing explicit keyboard shortcut instructions directly in the empty state text reduces cognitive load.
**Action:** Always include actionable instructions (e.g., "Press r to refresh") inside the empty state components of data grids or lists.
## 2026-08-18 - Missing empty state causes out-of-bounds panics
**Learning:** In TUI applications using Ratatui, rendering detail panes or context-specific keybindings based on a selected index into a data array can cause silent crashes (panics) if the data array is empty. Furthermore, showing contextual keybindings (like "Select" or "Edit") when there is nothing to select or edit is confusing to users.
**Action:** Always wrap the rendering of detail panes and contextual keybindings with an `!is_empty()` check on the backing dataset. When empty, explicitly render a dedicated empty state paragraph to prevent panics and clarify the application state to the user.
## 2026-08-26 - Provide visual feedback on disabled inputs
**Learning:** In TUI applications, modal prompts for numeric values when editing a quota must gracefully handle non-numeric inputs.
**Action:** Always intercept unhandled non-numeric typing.
## 2026-08-26 - Provide visual feedback on disabled inputs
**Learning:** In TUI applications, always wrap Table rendering in an is_empty check and render a dedicated empty state Paragraph instead.
**Action:** When a table is empty, do not render it, show an empty state.
## 2026-08-26 - Misleading values wrapped in brackets
**Learning:** In TUI applications, values wrapped in neutral brackets like '[ ]' are often interpreted as static info, whereas '< >' brackets clearly signal left/right cycleability or navigational affordance.
**Action:** Use '< >' for values that support horizontal cycle navigation instead of '[ ]' to prevent user confusion.
## 2024-09-07 - Actionable CTAs in Empty States implementation
**Learning:** Adding actionable CTAs (e.g., "Press 'r' to force refresh.") to empty states across all tables (Summary, Agents, Sessions, Quotas) drastically improves discoverability of actions when users feel stuck at dead-ends.
**Action:** Always include actionable instructions directly inside the empty state components of data grids or lists to reduce cognitive load.
