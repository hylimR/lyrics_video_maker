---
trigger: always_on
---

# Vite React Project: Code Style & Best Practices

This guide outlines the standards for keeping our code clean, readable, and easy to maintain.

## 1. Folder Structure
Organize files by what they *do*, not just by file type.

* **`src/components/`**: Reusable building blocks (Buttons, Navbars, Cards).
* **`src/pages/`**: Full application screens (Home, About, Login).
* **`src/hooks/`**: Custom logic reused across different components.
* **`src/utils/`**: Helper functions (math, date formatting) that don't use React.
* **`src/assets/`**: Images, global fonts, and icons.

## 2. Naming Conventions
Naming things consistently helps us identify files instantly.

| Type | Style | Example |
| :--- | :--- | :--- |
| **Components** | PascalCase (Capitalize every word) | `UserProfile.jsx` |
| **Functions/Vars** | camelCase (Start small, then capital) | `fetchUserData` |
| **Constants** | UPPER_SNAKE_CASE (All caps) | `MAX_COUNT` |
| **Custom Hooks** | camelCase (Must start with 'use') | `useWindowSize.js` |

## 3. Component Rules
* **One Component Per File:** Avoid putting multiple components in a single file unless they are tiny and never used elsewhere.
* **Destructure Props:** Unlock your variables at the top of the function.
    * **Bad:** `<div>{props.user.name}</div>`
    * **Good:** `const { user } = props;` or `({ user })`
* **Keep it Simple:** If a component exceeds 150 lines, try to break it down into smaller sub-components.

## 4. Imports
Use **Absolute Imports** to keep paths clean. Avoid long chains of dots.

* **Bad:** `import Button from '../../../components/Button'`
* **Good:** `import Button from '@/components/Button'`

## 5. CSS & Styling
Avoid using one massive CSS file. Use one of these distinct strategies:
* **CSS Modules:** Files named `Component.module.css` (prevents style clashes).
* **Tailwind CSS:** Utility classes written directly in the JSX (e.g., `className="p-4 bg-blue-500"`).

## 6. Automation
Let the computer fix your code style so you don't have to worry about it.
* **ESLint:** Finds errors (like unused variables).
* **Prettier:** Fixes formatting (spacing, commas, semicolons) automatically when you save.