export type ArtworkSlotRegistration = {
  id: string;
  cacheKey: string;
  artistSort: string;
  titleSort: string;
};

export type ArtworkSlotRecord = ArtworkSlotRegistration & {
  element: HTMLElement;
};

const slotRegistry = new Map<string, ArtworkSlotRecord>();
const listeners = new Set<() => void>();

function notifyListeners(): void {
  for (const listener of listeners) {
    listener();
  }
}

function sameRegistration(a: ArtworkSlotRegistration, b: ArtworkSlotRegistration): boolean {
  return (
    a.id === b.id &&
    a.cacheKey === b.cacheKey &&
    a.artistSort === b.artistSort &&
    a.titleSort === b.titleSort
  );
}

function upsertSlot(node: HTMLElement, registration: ArtworkSlotRegistration): void {
  const existing = slotRegistry.get(registration.id);
  if (existing && existing.element === node && sameRegistration(existing, registration)) {
    return;
  }

  slotRegistry.set(registration.id, {
    ...registration,
    element: node
  });
  notifyListeners();
}

function removeSlot(node: HTMLElement, id: string): void {
  const existing = slotRegistry.get(id);
  if (!existing || existing.element !== node) return;

  slotRegistry.delete(id);
  notifyListeners();
}

export function subscribeArtworkSlots(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function getArtworkSlotsSnapshot(): Map<string, ArtworkSlotRecord> {
  return new Map(slotRegistry);
}

export function registerArtworkSlot(node: HTMLElement, initial: ArtworkSlotRegistration) {
  let registration = initial;
  upsertSlot(node, registration);

  return {
    update(next: ArtworkSlotRegistration) {
      if (registration.id !== next.id) {
        removeSlot(node, registration.id);
      }
      registration = next;
      upsertSlot(node, registration);
    },
    destroy() {
      removeSlot(node, registration.id);
    }
  };
}
