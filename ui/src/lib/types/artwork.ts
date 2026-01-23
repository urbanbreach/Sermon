// Artwork API Types (Milestone 06)

export type ArtworkSource = 'trackOverride' | 'albumSelection' | 'embedded' | 'folder' | 'cache' | 'none';

export interface BestArtworkResponse {
  source: ArtworkSource;
  cacheKey?: string;
  mime?: string;
}

export interface ArtworkBytesResponse {
  mime: string;
  bytesBase64: string;
}

export interface ArtworkCandidate {
  provider: string;
  providerItemId: string;
  imageUrl: string;
}

export interface SearchCandidatesResponse {
  candidates: ArtworkCandidate[];
}

export interface SelectCandidateResponse {
  cacheKey: string;
  cacheHit: boolean;
}


export interface FindFolderArtworkResponse {
  found: boolean;
  cacheKey?: string;
  mime?: string;
  filename?: string;
}
