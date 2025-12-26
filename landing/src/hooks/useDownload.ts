import { useState, useEffect, useCallback } from "react";

type Platform = "mac-arm" | "mac-intel" | "windows" | "linux" | "unknown";

interface ReleaseAsset {
  name: string;
  browser_download_url: string;
}

interface GitHubRelease {
  tag_name: string;
  assets: ReleaseAsset[];
}

interface DownloadInfo {
  platform: Platform;
  platformLabel: string;
  downloadUrl: string | null;
  isSupported: boolean;
  isLoading: boolean;
  error: string | null;
  version: string | null;
}

const GITHUB_REPO = "vitalii-zinchenko/dictara";
const RELEASES_URL = `https://github.com/${GITHUB_REPO}/releases`;

function detectPlatform(): Platform {
  const userAgent = navigator.userAgent.toLowerCase();
  const platform = navigator.platform.toLowerCase();

  // Check for macOS
  if (platform.includes("mac") || userAgent.includes("mac")) {
    // Detect Apple Silicon using WebGL renderer info
    // Apple Silicon GPUs contain "Apple" in the renderer string
    const canvas = document.createElement("canvas");
    const gl = canvas.getContext("webgl");
    const debugInfo = gl?.getExtension("WEBGL_debug_renderer_info");
    const renderer = debugInfo
      ? gl?.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL)
      : "";

    if (
      renderer &&
      typeof renderer === "string" &&
      renderer.toLowerCase().includes("apple")
    ) {
      return "mac-arm";
    }

    // If no "Apple" GPU detected, it's likely an Intel Mac
    // (Intel Macs have Intel HD/Iris or AMD GPUs)
    return "mac-intel";
  }

  if (platform.includes("win") || userAgent.includes("windows")) {
    return "windows";
  }

  if (platform.includes("linux") || userAgent.includes("linux")) {
    return "linux";
  }

  return "unknown";
}

function getPlatformLabel(platform: Platform): string {
  switch (platform) {
    case "mac-arm":
      return "macOS (Apple Silicon)";
    case "mac-intel":
      return "macOS (Intel)";
    case "windows":
      return "Windows";
    case "linux":
      return "Linux";
    default:
      return "your platform";
  }
}

function findAssetForPlatform(
  assets: ReleaseAsset[],
  platform: Platform
): ReleaseAsset | null {
  // Look for the appropriate asset based on platform
  for (const asset of assets) {
    const name = asset.name.toLowerCase();

    if (platform === "mac-arm") {
      // Match aarch64/arm64 DMG files
      if (
        (name.includes("aarch64") || name.includes("arm64")) &&
        name.endsWith(".dmg")
      ) {
        return asset;
      }
    }

    if (platform === "mac-intel") {
      // Match x64/intel DMG files (when available)
      if (
        (name.includes("x64") ||
          name.includes("x86_64") ||
          name.includes("intel")) &&
        name.endsWith(".dmg")
      ) {
        return asset;
      }
    }

    if (platform === "windows") {
      // Match Windows installers
      if (name.endsWith(".exe") || name.endsWith(".msi")) {
        return asset;
      }
    }

    if (platform === "linux") {
      // Match Linux packages
      if (
        name.endsWith(".deb") ||
        name.endsWith(".appimage") ||
        name.endsWith(".rpm")
      ) {
        return asset;
      }
    }
  }

  return null;
}

export function useDownload(): DownloadInfo & { download: () => void } {
  const [platform] = useState<Platform>(() => detectPlatform());
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [version, setVersion] = useState<string | null>(null);
  const [isSupported, setIsSupported] = useState(false);

  useEffect(() => {
    async function fetchLatestRelease() {
      try {
        const response = await fetch(
          `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`
        );

        if (!response.ok) {
          throw new Error("Failed to fetch release info");
        }

        const release: GitHubRelease = await response.json();
        setVersion(release.tag_name);

        const asset = findAssetForPlatform(release.assets, platform);

        if (asset) {
          setDownloadUrl(asset.browser_download_url);
          setIsSupported(true);
        } else {
          setIsSupported(false);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : "Unknown error");
        setIsSupported(false);
      } finally {
        setIsLoading(false);
      }
    }

    fetchLatestRelease();
  }, [platform]);

  const download = useCallback(() => {
    if (downloadUrl) {
      window.open(downloadUrl, "_blank");
    } else {
      // Fallback to releases page
      window.open(RELEASES_URL, "_blank");
    }
  }, [downloadUrl]);

  return {
    platform,
    platformLabel: getPlatformLabel(platform),
    downloadUrl,
    isSupported,
    isLoading,
    error,
    version,
    download,
  };
}
