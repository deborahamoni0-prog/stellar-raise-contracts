# ReactSubmitButton

A typed React submit button with a strict state machine, safe label handling,
double-submit prevention, and ARIA accessibility — designed for Stellar/Soroban
transaction flows.

## States

| State        | Description                                              | Interaction |
| :----------- | :------------------------------------------------------- | :---------- |
| `idle`       | Default; ready to submit                                 | Clickable   |
| `submitting` | Transaction in-flight; blocks duplicate submissions      | Blocked     |
| `success`    | Transaction confirmed                                    | Blocked     |
| `error`      | Transaction failed; user may retry                       | Clickable   |
| `disabled`   | Externally disabled (deadline passed, goal met, etc.)    | Blocked     |

### Allowed Transitions

```
idle        → submitting | disabled
submitting  → success | error | disabled
success     → idle | disabled
error       → idle | submitting | disabled
disabled    → idle
```

Invalid transitions in strict mode fall back to `previousState`.

## Props

| Prop                | Type                        | Default      | Description                                      |
| :------------------ | :-------------------------- | :----------- | :----------------------------------------------- |
| `state`             | `SubmitButtonState`         | —            | Current button state (required)                  |
| `previousState`     | `SubmitButtonState`         | `undefined`  | Used for strict transition validation             |
| `strictTransitions` | `boolean`                   | `true`       | Enforce the allowed-transition map                |
| `labels`            | `SubmitButtonLabels`        | `undefined`  | Per-state label overrides                         |
| `onClick`           | `(e) => void \| Promise`    | `undefined`  | Async-safe handler; blocked while submitting      |
| `type`              | `"button" \| "submit" \| "reset"` | `"button"` | HTML button type                           |
| `disabled`          | `boolean`                   | `undefined`  | External disabled override                        |
| `className`         | `string`                    | `undefined`  | Additional CSS class                              |
| `id`                | `string`                    | `undefined`  | HTML id attribute                                 |

## Usage

```tsx
import ReactSubmitButton from "./react_submit_button";

// Basic
<ReactSubmitButton state="idle" onClick={handleContribute} />

// With state machine enforcement
<ReactSubmitButton
  state={txState}          // e.g. "submitting"
  previousState={prevState} // e.g. "idle"
  strictTransitions
  labels={{ idle: "Fund Campaign", submitting: "Funding…", success: "Funded!" }}
  onClick={handleContribute}
/>
```

## Exported Helpers

All pure helpers are exported for independent unit testing:

| Export                                  | Purpose                                              |
| :-------------------------------------- | :--------------------------------------------------- |
| `normalizeSubmitButtonLabel`            | Sanitize/truncate a label candidate                  |
| `resolveSubmitButtonLabel`              | Pick the correct label for a given state             |
| `isValidSubmitButtonStateTransition`    | Check if a state transition is allowed               |
| `resolveSafeSubmitButtonState`          | Apply strict-mode fallback logic                     |
| `isSubmitButtonInteractionBlocked`      | Determine if clicks should be suppressed             |
| `isSubmitButtonBusy`                    | Determine if `aria-busy` should be `true`            |
| `ALLOWED_TRANSITIONS`                   | The canonical transition map (shared with tests)     |

## Security Assumptions

- **No `dangerouslySetInnerHTML`** — labels are rendered as React text nodes only.
- **Label sanitization** — control characters are stripped; labels are truncated at
  80 characters to prevent layout abuse.
- **Double-submit prevention** — the component maintains a local `isLocallySubmitting`
  flag that blocks re-entry while an async `onClick` is in-flight, preventing
  duplicate Stellar transactions.
- **Hardcoded styles** — all colours are compile-time constants; no user input
  reaches CSS properties.
- **Caller responsibility** — input validation (amounts, addresses) must be
  performed by the parent before calling `onClick`.

## Testing

```bash
# Run the full test suite
npm test -- --testPathPattern=react_submit_button

# With coverage
npm run test:coverage -- --testPathPattern=react_submit_button
```

Coverage target: ≥ 95% (branches, lines, functions, statements).

The test suite covers:

- `normalizeSubmitButtonLabel` — non-string inputs, empty/whitespace, control
  characters, 80-char boundary, truncation, XSS-like strings
- `resolveSubmitButtonLabel` — all five states, custom overrides, fallback logic
- `isValidSubmitButtonStateTransition` — every allowed edge, every blocked edge,
  same-state idempotency
- `resolveSafeSubmitButtonState` — strict/non-strict modes, missing previousState
- `isSubmitButtonInteractionBlocked` — all blocking conditions
- `isSubmitButtonBusy` — submitting state, local flag
- Component rendering — all states, props, ARIA attributes
- Click handling — idle/error fire, submitting/disabled/success block,
  async handler, rejected promise, no-onClick guard
- Strict transition enforcement — invalid fallback, valid pass-through, non-strict
