## 2024-05-23 - Replaced Mock Data with Empty States
**Learning:** Hardcoded mock data can be confusing and alarming in quota/usage tracking tools. Users might interpret mock data as rogue usage or bugs. Displaying fake sessions in the recent sessions tab caused potential user anxiety.
**Action:** Always implement a dedicated "empty state" component with helpful copy (e.g. "No recent sessions found") when data sets are empty, rather than inserting illustrative dummy data.

## 2024-05-24 - Improve keyboard navigation discoverability and accurate keybinds
**Learning:** Terminal applications with hidden keybinds lead to user confusion. Incorrect keybind hints lead to frustration when standard controls (like left/right arrows) map to unexpected actions (global tab switching instead of cycling local values). Context-aware hints (like hiding an edit hint for an uninstalled agent) improve UX by preventing users from attempting invalid actions.
**Action:** Always expose available hotkeys in titles (like tab numbers). Double-check input logic vs instructional text, and dynamically hide hints for unavailable actions.
