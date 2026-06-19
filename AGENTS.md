# AGENTS.md — Codex agent instructions for evocode-tauri

## UI Component Rules

- **All UI components MUST use ant-design-vue** (`<a-*>`) components.
- **DO NOT** write custom HTML/CSS components, hand-made sliders, custom drag logic, or raw `<div>` based controls when an ant-design-vue equivalent exists.
- Use `<a-slider>` for sliders, `<a-select>` for dropdowns, `<a-input>` for text inputs, `<a-button>` for buttons, `<a-modal>` for modals, `<a-tabs>` for tabs, `<a-form>` for forms, `<a-switch>` for toggles, etc.
- Stick to Ant Design Vue v4 API and components.

## Code Style

- Vue 3 Composition API with `<script setup lang="ts">`
- Use `ref` / `reactive` for state
- Use `defineExpose` when exposing to parent
- i18n: use `useLocale()` + `t()`
- TypeScript strict mode

## Project Structure

- `src/views/` — page-level components
- `src/views/config_view/` — config/settings panels  
- `src/api/` — Tauri invoke wrappers
- `src/stores/` — Pinia stores
- `src/components/` — shared components
- `src/composables/` — shared composition functions
- `src-tauri/src/` — Rust backend

## Validation Checklist

- After completing any functional change, run `npx vue-tsc --noEmit && npx vite build` to verify no TypeScript or build errors.
- Fix any reported errors before considering the change done.
