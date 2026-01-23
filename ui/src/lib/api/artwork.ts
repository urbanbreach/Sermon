import { invoke } from '@tauri-apps/api/core';
import type {
  BestArtworkResponse,
  ArtworkBytesResponse,
  SearchCandidatesResponse,
  SelectCandidateResponse,
  FindFolderArtworkResponse,
} from '../types/artwork';

export async function getArtworkBestForAlbum(
  albumArtistSort: string,
  albumTitleSort: string
): Promise<BestArtworkResponse> {
  return invoke('cmd_artwork_get_best_for_album', {
    request: { albumArtistSort, albumTitleSort },
  });
}

export async function getArtworkBestForTrack(trackId: number): Promise<BestArtworkResponse> {
  return invoke('cmd_artwork_get_best_for_track', { request: { trackId } });
}

export async function getArtworkBytes(cacheKey: string, mime: string): Promise<ArtworkBytesResponse> {
  return invoke('cmd_artwork_get_bytes', { request: { cacheKey, mime } });
}

export async function searchArtworkCandidates(
  albumArtist?: string,
  albumTitle?: string,
  trackTitle?: string
): Promise<SearchCandidatesResponse> {
  return invoke('cmd_artwork_search_candidates', {
    request: { albumArtist, albumTitle, trackTitle },
  });
}

export async function selectArtworkCandidateForAlbum(
  albumArtistSort: string,
  albumTitleSort: string,
  provider: string,
  providerItemId: string,
  imageUrl: string
): Promise<SelectCandidateResponse> {
  return invoke('cmd_artwork_select_candidate_for_album', {
    request: { albumArtistSort, albumTitleSort, provider, providerItemId, imageUrl },
  });
}

export interface EmbedArtworkResponse {
  success: boolean;
}

export async function embedArtworkToFile(
  trackId: number,
  cacheKey: string,
  mime: string
): Promise<EmbedArtworkResponse> {
  return invoke('cmd_artwork_embed_to_file', {
    request: { trackId, cacheKey, mime },
  });
}


/**
 * Find artwork in the same folder as the track (cover.jpg, folder.jpg, etc.)
 */
export async function findFolderArtwork(trackId: number): Promise<FindFolderArtworkResponse> {
  return invoke('cmd_artwork_find_folder', {
    request: { trackId },
  });
}
