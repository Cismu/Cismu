export interface Track {
    id: number;
    path: string;
    file: FileInfo;
    tags: TagInfo;
    audio: AudioInfo;
  }
  
  export interface FileInfo {
    filename: string;
    size_bytes: number;
    modified: number;
  }
  
  export interface TagInfo {
    title: string | null;
    artist: string | null;
    album: string | null;
    album_artist: string | null;
    track_number: number | null;
    total_tracks: number | null;
    disc_number: number | null;
    total_discs: number | null;
    genre: string | null;
    year: number | null;
    composer: string | null;
    publisher: string | null;
    comments: string | null;
    artwork: Artwork[] | null;
    rating: Rating;
  }
  
  export interface Artwork {
    data: number[];
    mime_type: string;
    description: string;
  }
  
  export interface AudioInfo {
    duration_secs: Duration;
    bitrate_kbps: number | null;
    sample_rate_hz: number | null;
    channels: number | null;
    quality_score: number | null;
    analysis: AudioAnalysis | null;
    tag_type: string | null;
  }
  
  export interface AudioAnalysis {
    spectral_analysis: AnalysisOutcome;
    quality_score: number;
    overall_assessment: string;
  }
  
  /** Coincide con la serialización externa de tu enum `AnalysisOutcome` */
  export type AnalysisOutcome =
    | { CutoffDetected: {
        cutoff_frequency_hz: number;
        reference_level_db: number;
        cutoff_band_level_db: number;
      }}
    | { NoCutoffDetected: {
        reference_level_db: number;
        max_analyzed_freq_hz: number;
      }}
    | { InconclusiveNotEnoughWindows: {
        processed_windows: number;
        required_windows: number;
      }}
    | { InconclusiveReferenceBandError: {} }
    | { InconclusiveLowReferenceLevel: {
        reference_level_db: number;
      }}
    | { InconclusiveError: {} };
  
  /** Coincide con la serialización externa de tu enum `Rating` */
  export type Rating =
    | 'Unrated'
    | { Stars: number };
  
  /** Para mapear `std::time::Duration` serializado como `{ secs: number, nanos: number }` */
  export interface Duration {
    secs: number;
    nanos: number;
  }
  