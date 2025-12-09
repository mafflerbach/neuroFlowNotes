/**
 * File type detection utilities
 * Centralized logic for determining file types based on extensions
 */

// Supported file extensions by category
export const IMAGE_EXTENSIONS = ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico"] as const;
export const AUDIO_EXTENSIONS = ["mp3", "wav", "ogg", "m4a", "flac", "aac"] as const;
export const VIDEO_EXTENSIONS = ["mp4", "webm", "mov", "avi"] as const;
export const PDF_EXTENSIONS = ["pdf"] as const;
export const MARKDOWN_EXTENSIONS = ["md", "markdown"] as const;

// Type definitions
export type ImageExtension = typeof IMAGE_EXTENSIONS[number];
export type AudioExtension = typeof AUDIO_EXTENSIONS[number];
export type VideoExtension = typeof VIDEO_EXTENSIONS[number];
export type PdfExtension = typeof PDF_EXTENSIONS[number];
export type MarkdownExtension = typeof MARKDOWN_EXTENSIONS[number];
export type MediaExtension = ImageExtension | AudioExtension | VideoExtension | PdfExtension;

export type FileType = "image" | "audio" | "video" | "pdf" | "markdown" | "unknown";

/**
 * Extract file extension from a path or filename
 */
export function getFileExtension(filePath: string): string {
  return filePath.split(".").pop()?.toLowerCase() ?? "";
}

/**
 * Check if file is an image
 */
export function isImageFile(filePath: string): boolean {
  return (IMAGE_EXTENSIONS as readonly string[]).includes(getFileExtension(filePath));
}

/**
 * Check if file is audio
 */
export function isAudioFile(filePath: string): boolean {
  return (AUDIO_EXTENSIONS as readonly string[]).includes(getFileExtension(filePath));
}

/**
 * Check if file is video
 */
export function isVideoFile(filePath: string): boolean {
  return (VIDEO_EXTENSIONS as readonly string[]).includes(getFileExtension(filePath));
}

/**
 * Check if file is PDF
 */
export function isPdfFile(filePath: string): boolean {
  return (PDF_EXTENSIONS as readonly string[]).includes(getFileExtension(filePath));
}

/**
 * Check if file is markdown
 */
export function isMarkdownFile(filePath: string): boolean {
  return (MARKDOWN_EXTENSIONS as readonly string[]).includes(getFileExtension(filePath));
}

/**
 * Check if file is any supported media type (image, audio, video, PDF)
 */
export function isMediaFile(filePath: string): boolean {
  return isImageFile(filePath) || isAudioFile(filePath) || isVideoFile(filePath) || isPdfFile(filePath);
}

/**
 * Get the type of a file
 */
export function getFileType(filePath: string): FileType {
  if (isMarkdownFile(filePath)) return "markdown";
  if (isImageFile(filePath)) return "image";
  if (isAudioFile(filePath)) return "audio";
  if (isVideoFile(filePath)) return "video";
  if (isPdfFile(filePath)) return "pdf";
  return "unknown";
}

/**
 * Get MIME type for audio files
 */
export function getAudioMimeType(filePath: string): string {
  const ext = getFileExtension(filePath);
  const mimeTypes: Record<string, string> = {
    mp3: "audio/mpeg",
    wav: "audio/wav",
    ogg: "audio/ogg",
    m4a: "audio/mp4",
    flac: "audio/flac",
    aac: "audio/aac",
  };
  return mimeTypes[ext] || "audio/mpeg";
}

/**
 * Get MIME type for video files
 */
export function getVideoMimeType(filePath: string): string {
  const ext = getFileExtension(filePath);
  const mimeTypes: Record<string, string> = {
    mp4: "video/mp4",
    webm: "video/webm",
    mov: "video/quicktime",
    avi: "video/x-msvideo",
  };
  return mimeTypes[ext] || "video/mp4";
}
