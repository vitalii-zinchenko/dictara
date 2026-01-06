# Context

To learn more about open source libraries we use you can use the Context7, WebSearch, node_modules or the source code in `.ai-repos`.
Also, you can clone source code for other libraries if you need so.

# Code changes workflow:

IMPORTANT: After you make changes you must run `npm run verify` to check if there are any errors, and fix all errors and warnings.

# Frontend

## UI

The UI is built with [Shadcn](https://github.com/shadcn/ui) and [React](https://reactjs.org/).
Shadcn components are located in `src/components/ui` and new components can be added by using the `shadcn` cli: `npx shadcn@latest add [component]`.

## Frontend to Backend Communication

This project uses `tauri-specta` to generate type-safe bindings for the frontend to the backend. The bindings are located in `src/bindings.ts` and are generated automatically if the rust code mapped accordingly using specta macros. For Get/Modify/Delete operations the project uses TanStack Queries located in `src/hooks` (you need to create a new hook for each operation) and hooks are implemented using the specta bindings.

Use cases:

- Get/Modify/Delete operations:
  - use TanStack Queries from `src/hooks`
- Events:
  - use specta bindings from `src/bindings.ts`

IMPORTANT: You must never use the the tauri's invoke/listen methods directly in the frontend. Use the bindings/ts-hooks instead.
