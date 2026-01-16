# Two-Layer Hooks Architecture Refactoring

## Overview

Refactor the hooks folder to separate TanStack Query boilerplate hooks from custom/composite hooks. This creates a clear two-layer architecture that will enable future code generation via Rust macros.

## Why This Design Is Better

### 1. Clear Separation of Concerns
- **Layer 1 (TanStack Boilerplate)**: Simple wrappers around Tauri Specta commands
- **Layer 2 (Custom Hooks)**: Business logic, composed behaviors, complex state management

### 2. Future-Proof for Code Generation
When Rust macros are created to auto-generate TanStack hooks from Specta definitions:
- Clean target folder: `hooks/tanstack/`
- Generated files won't mix with hand-written custom hooks
- Easy to identify which files should never be manually edited

### 3. Better Discoverability
Developers instantly know where to look:
- Need a basic query/mutation? → `hooks/tanstack/`
- Need composed business logic? → `hooks/`
- Looking for UI utilities? → `hooks/` (non-TanStack)

### 4. Easier Maintenance
- Won't accidentally modify generated boilerplate
- Clear boundaries between auto-generated and hand-written code
- Simpler code reviews (boilerplate vs. business logic)

### 5. Scalability
- As the app grows, prevents the hooks folder from becoming overwhelming
- Logical grouping makes navigation easier
- Reduces cognitive load when searching for the right hook

## New Structure

```
src/hooks/
├── tanstack/                    # NEW: TanStack Query boilerplate
│   ├── index.ts                 # Barrel export (optional)
│   ├── useAppConfig.ts
│   ├── useSaveAppConfig.ts
│   ├── useShortcutsConfig.ts
│   ├── useSaveShortcutsConfig.ts
│   ├── useResetShortcutsConfig.ts
│   ├── useOnboardingConfig.ts
│   ├── useCheckForUpdates.ts
│   ├── useAccessibilityPermission.ts
│   ├── useMicrophonePermission.ts
│   ├── useRecording.ts
│   ├── useAzureOpenAIConfig.ts   # (needs verification)
│   ├── useOpenAIConfig.ts        # (needs verification)
│   └── useLocalModels.ts         # (needs verification)
│
├── useKeyCapture.ts             # Custom hook (stays here)
├── useOnboardingNavigation.ts   # Custom hook (stays here)
└── use-mobile.tsx               # UI utility (stays here)
```

## File Categorization

### ⚡ Simple TanStack Boilerplate (Move to `hooks/tanstack/`)

These are straightforward wrappers - simple queries or mutations with minimal logic:

1. **useAppConfig.ts** - Query hook for loading app config
2. **useSaveAppConfig.ts** - Mutation hook with cache invalidation
3. **useShortcutsConfig.ts** - Query hook for shortcuts config
4. **useSaveShortcutsConfig.ts** - Mutation hook with cache invalidation
5. **useResetShortcutsConfig.ts** - Mutation hook with cache invalidation
6. **useOnboardingConfig.ts** - Query hook for onboarding config
7. **useCheckForUpdates.ts** - Mutation hook with optional params
8. **useAccessibilityPermission.ts** - Contains 2 hooks (1 query + 1 mutation)
9. **useMicrophonePermission.ts** - Contains 3 hooks (1 query with polling + 2 mutations)
10. **useRecording.ts** - Contains 5 mutation hooks (all simple)

### 🧩 Complex/Custom Hooks (Keep in `hooks/`)

These have business logic, composed behaviors, or complex state management:

1. **useKeyCapture.ts**
   - Complex: useState, useEffect, event listeners
   - Manages key capture state and auto-finish callbacks
   - Business logic for preventing duplicates and max key limits
   - Not a simple TanStack wrapper

2. **useOnboardingNavigation.ts**
   - Complex: navigation orchestration, step management
   - Contains utility functions (stepToRoute, getNextStep, getPreviousStep, etc.)
   - Composes multiple mutations into cohesive navigation API
   - Uses TanStack internally but adds significant business logic
   - Also exports `useRestartOnboarding` (standalone mutation)

### 🎨 Non-TanStack Hooks (Keep in `hooks/`)

1. **use-mobile.tsx** - UI utility hook, not related to TanStack Query

### ❓ Needs Verification

These files need to be reviewed during implementation to determine category:

1. **useAzureOpenAIConfig.ts** - Likely simple TanStack boilerplate
2. **useOpenAIConfig.ts** - Likely simple TanStack boilerplate
3. **useLocalModels.ts** - Likely simple TanStack boilerplate

## Implementation Steps

### Phase 1: Setup
1. Create `src/hooks/tanstack/` folder
2. (Optional) Create `src/hooks/tanstack/index.ts` barrel export

### Phase 2: Move Simple Files
Move the 10 simple boilerplate files to `hooks/tanstack/`:
- useAppConfig.ts
- useSaveAppConfig.ts
- useShortcutsConfig.ts
- useSaveShortcutsConfig.ts
- useResetShortcutsConfig.ts
- useOnboardingConfig.ts
- useCheckForUpdates.ts
- useAccessibilityPermission.ts
- useMicrophonePermission.ts
- useRecording.ts

### Phase 3: Update Imports
Update all component imports across the codebase:
- From: `@/hooks/useAppConfig`
- To: `@/hooks/tanstack/useAppConfig`

Search for all imports from `@/hooks/use*` and update accordingly.

### Phase 4: Verification Files
Review and categorize the 3 unverified files:
- useAzureOpenAIConfig.ts
- useOpenAIConfig.ts
- useLocalModels.ts

Move them to `tanstack/` if they're simple boilerplate, or keep them in `hooks/` if they're complex.

### Phase 5: Testing
1. Run `npm run verify` to ensure no errors
2. Test that all hooks still work correctly
3. Verify imports are resolved properly

## 🚨 Important Notes for Implementer

### ⚠️ CRITICAL: Review Files Before Implementation

**This plan is based on the codebase state as of 2026-01-16.**

Before implementing, the implementer MUST:

1. **Re-audit the hooks folder** - New files may have been added, existing files may have been modified or removed
2. **Read each file** - Don't blindly move files based on this plan
3. **Verify categorization** - Confirm each file is truly simple boilerplate vs. custom logic
4. **Check for new patterns** - Look for any new hook patterns not covered in this plan
5. **Review the 3 unverified files** mentioned above

### Decision Criteria

**Move to `tanstack/` if:**
- Simple wrapper around a single Tauri command
- Uses only `useQuery` or `useMutation` with minimal logic
- Only performs cache invalidation in callbacks
- No complex state management or business logic

**Keep in `hooks/` if:**
- Uses useState, useEffect, or other React hooks
- Composes multiple queries/mutations
- Contains business logic, validation, or transformations
- Manages complex state or event listeners
- Not related to TanStack Query at all

## Expected Outcome

After this refactoring:

1. ✅ Clear separation between boilerplate and custom hooks
2. ✅ Ready for future Rust macro code generation
3. ✅ Easier to navigate and maintain
4. ✅ Better developer experience
5. ✅ All tests passing, no functionality broken

## Migration Path for Future Code Generation

Once Rust macros are implemented:

1. Macros generate hooks directly into `hooks/tanstack/`
2. Delete manually-written boilerplate files
3. Custom hooks in `hooks/` continue to use generated hooks
4. Clear separation prevents conflicts between generated and hand-written code

---

**Plan Created:** 2026-01-16
**Status:** Pending Implementation
**Estimated Complexity:** Medium (mostly mechanical refactoring, but requires careful import updates)
