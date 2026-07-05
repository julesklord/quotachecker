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
