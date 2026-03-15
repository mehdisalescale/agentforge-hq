STATUS: COMPLETE
COMPONENTS_CREATED: [focusTrap.ts]
PAGES_UPDATED: [+layout.svelte, app.html, app.css, companies, personas, goals, approvals, agents]
PNPM_CHECK: pass (1 pre-existing error in workflows/+page.svelte, 0 new errors)
PNPM_BUILD: pass
NOTES:
- Skip-to-content link added to app.html (sr-only, visible on focus)
- SR utilities (.sr-only, .sr-only-focusable) and :focus-visible styles added to app.css
- Layout: nav aria-label="Main navigation", main id="main-content", footer role="status", aria-current on all nav links
- focusTrap Svelte action created in $lib/actions/focusTrap.ts — traps Tab within modal, auto-focuses first element
- Modals in companies, personas, goals, agents pages: use:focusTrap on .modal div, Escape key handler via onkeydown on .modal-backdrop, tabindex="-1" for a11y compliance
- Approval status badges: role="status" + aria-label="Status: {status}"
- aria-busy={loading} on page wrapper divs (companies, personas, goals, approvals, agents)
- <svelte:window> cannot be inside {#if} blocks in Svelte 5 — used onkeydown on modal-backdrop instead
