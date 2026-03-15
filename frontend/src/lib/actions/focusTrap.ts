/**
 * Svelte action that traps focus within an element (for modals).
 * Usage: <div use:focusTrap>
 */
export function focusTrap(node: HTMLElement) {
  const focusable = 'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

  function handleKeydown(e: KeyboardEvent) {
    if (e.key !== 'Tab') return;

    const elements = Array.from(node.querySelectorAll(focusable)) as HTMLElement[];
    if (elements.length === 0) return;

    const first = elements[0];
    const last = elements[elements.length - 1];

    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }

  // Focus first focusable element on mount
  requestAnimationFrame(() => {
    const elements = node.querySelectorAll(focusable) as NodeListOf<HTMLElement>;
    if (elements.length > 0) elements[0].focus();
  });

  node.addEventListener('keydown', handleKeydown);

  return {
    destroy() {
      node.removeEventListener('keydown', handleKeydown);
    }
  };
}
