export interface Album {
  id: string;
  title: string;
  artistId: string;
  year: number;
  trackIds: string[];
  artworkFile: string;
}

export interface Artist {
  id: string;
  name: string;
}

export interface Track {
  id: string;
  title: string;
  albumId: string;
  artistId: string;
  durationMs: number;
  trackNumber: number;
  discNumber: number;
}

export interface LibraryData {
  artists: Artist[];
  albums: Album[];
  tracks: Track[];
}

import libraryData from '../../../fixtures/library.json';

const data = libraryData as LibraryData;

// Use import.meta.glob to get URLs for the artwork
const artworks = import.meta.glob('../../../fixtures/artwork/*.{jpg,png,webp,svg}', { eager: true, as: 'url' });

export const Fixtures = {
  getAlbums: (): Album[] => {
    return data.albums;
  },
  
  getArtists: (): Artist[] => {
    return data.artists;
  },
  
  getTracks: (): Track[] => {
    return data.tracks;
  },

  getAlbum: (id: string): Album | undefined => {
    return data.albums.find(a => a.id === id);
  },

  getArtist: (id: string): Artist | undefined => {
    return data.artists.find(a => a.id === id);
  },
  
  getTrack: (id: string): Track | undefined => {
    return data.tracks.find(t => t.id === id);
  },
  
  getTracksByAlbum: (albumId: string): Track[] => {
    return data.tracks.filter(t => t.albumId === albumId).sort((a, b) => {
      if (a.discNumber !== b.discNumber) return a.discNumber - b.discNumber;
      return a.trackNumber - b.trackNumber;
    });
  },

  getArtworkPath: (filename: string): string => {
    const key = Object.keys(artworks).find(k => k.endsWith(filename));
    return key ? artworks[key] : '';
  }
};
