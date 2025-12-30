/**
 * Waits for the browser to complete a paint cycle.
 *
 * This is useful when you need to ensure the UI has updated before
 * continuing with async operations. A single requestAnimationFrame
 * fires BEFORE the paint, so we use two to ensure the paint has completed.
 *
 * @see https://github.com/TanStack/form/issues/1967
 */
export const waitForPaint = () =>
  new Promise<void>((resolve) => {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => resolve())
    })
  })
