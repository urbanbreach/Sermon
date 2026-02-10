const playedEnterNonces = new Set<number>();
const MAX_TRACKED_NONCES = 4096;

export function shouldAnimateAlbumInlineEnter(animationNonce: number): boolean {
  if (animationNonce <= 0) {
    return true;
  }

  if (playedEnterNonces.has(animationNonce)) {
    return false;
  }

  if (playedEnterNonces.size > MAX_TRACKED_NONCES) {
    playedEnterNonces.clear();
  }

  playedEnterNonces.add(animationNonce);
  return true;
}
