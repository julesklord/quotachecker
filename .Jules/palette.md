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
