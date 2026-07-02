export const WEBOS_BACK_KEYCODE = 461;

/** True when the event target is (or is inside) a text entry control. */
export function isEditableElement(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  const element =
    target.closest('input, textarea, select, [contenteditable=""], [contenteditable="true"]') ??
    target;

  if (!(element instanceof HTMLElement)) {
    return false;
  }

  if (element.isContentEditable) {
    return true;
  }

  if (element instanceof HTMLInputElement) {
    const type = element.type.toLowerCase();
    return type !== 'button' && type !== 'submit' && type !== 'checkbox' && type !== 'radio';
  }

  return element instanceof HTMLTextAreaElement || element instanceof HTMLSelectElement;
}

/** Magic Remote / browser back — never treat Backspace as back on TV keyboards. */
export function isWebOsBackKey(event: KeyboardEvent): boolean {
  return (
    event.keyCode === WEBOS_BACK_KEYCODE ||
    event.key === 'GoBack' ||
    event.key === 'BrowserBack'
  );
}

/** Desktop dev fallback when testing without a Magic Remote. */
export function isDevBackKey(event: KeyboardEvent): boolean {
  return import.meta.env.DEV && event.key === 'Escape';
}

export function isAppBackKey(event: KeyboardEvent): boolean {
  return isWebOsBackKey(event) || isDevBackKey(event);
}

/** Skip global shortcuts while the user is typing. */
export function shouldDeferToTextInput(event: KeyboardEvent): boolean {
  return isEditableElement(event.target) || isEditableElement(document.activeElement);
}
