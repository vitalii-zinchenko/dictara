import type { SVGProps } from "react";

type IconProps = SVGProps<SVGSVGElement>;

// Lightning Fast - Stylized bolt with motion lines
export function IconLightning(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
      {/* Motion lines */}
      <path d="M1 7h4M1 12h3M1 17h4" stroke="currentColor" strokeWidth="2" strokeLinecap="round" opacity="0.6" />
      {/* Lightning bolt - bigger */}
      <path
        d="M14 1L7 12h5l-1 11 9-14h-6l3-8z"
        fill="currentColor"
        stroke="currentColor"
        strokeWidth="1"
        strokeLinejoin="round"
      />
    </svg>
  );
}

// 90+ Languages - Globe with speech elements
export function IconLanguages(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
      {/* Globe - bigger */}
      <circle cx="11" cy="12" r="10" stroke="currentColor" strokeWidth="1.5" />
      <ellipse cx="11" cy="12" rx="4.5" ry="10" stroke="currentColor" strokeWidth="1.5" />
      <path d="M1 12h20" stroke="currentColor" strokeWidth="1.5" />
      <path d="M2.5 6h17M2.5 18h17" stroke="currentColor" strokeWidth="1.5" opacity="0.6" />
      {/* Speech indicator - bigger */}
      <circle cx="19" cy="4" r="4" fill="currentColor" />
      {/* Letter A as path */}
      <path
        d="M19 1.5l-2 5h1l.3-.8h1.4l.3.8h1l-2-5zm0 1.2l.5 1.3h-1l.5-1.3z"
        fill="var(--background, #0f0a1a)"
      />
    </svg>
  );
}

// Privacy First - Shield with keyhole
export function IconPrivacy(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
      {/* Shield */}
      <path
        d="M12 2L3 6v5.5c0 5.5 3.5 10.5 9 12 5.5-1.5 9-6.5 9-12V6L12 2z"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinejoin="round"
      />
      {/* Keyhole */}
      <circle cx="12" cy="10" r="2.5" fill="currentColor" />
      <path d="M12 12.5v4.5" stroke="currentColor" strokeWidth="3" strokeLinecap="round" />
    </svg>
  );
}

// Powered by Whisper - Sound wave with AI sparkle
export function IconWhisper(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
      {/* Sound waves - bigger and bolder */}
      <path d="M2 12h2" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" />
      <path d="M5.5 7v10" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" />
      <path d="M9 3v18" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" />
      <path d="M12.5 6v12" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" />
      <path d="M16 9v6" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" />
      {/* AI sparkle - bigger */}
      <path d="M20 3l.7 2 2 .7-2 .7-.7 2-.7-2-2-.7 2-.7.7-2z" fill="currentColor" />
      <path d="M19.5 14l.5 1.5 1.5.5-1.5.5-.5 1.5-.5-1.5-1.5-.5 1.5-.5.5-1.5z" fill="currentColor" opacity="0.7" />
    </svg>
  );
}

// Simple Workflow - FN key with tap indicator
export function IconWorkflow(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
      {/* Key cap */}
      <rect x="2" y="7" width="20" height="15" rx="2.5" stroke="currentColor" strokeWidth="1.5" />
      <rect x="4" y="9" width="16" height="11" rx="1.5" stroke="currentColor" strokeWidth="1" opacity="0.5" />
      {/* FN as paths - F letter */}
      <path d="M7 12v6M7 12h3M7 15h2.5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      {/* N letter */}
      <path d="M13 18v-6l4 6v-6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      {/* Tap indicator */}
      <path d="M12 1v3" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
      <path d="M8 3l4-2 4 2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

// Auto Paste - Cursor with text appearing
export function IconAutoPaste(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
      {/* Text cursor */}
      <path d="M9 4v16" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
      <path d="M6 4h6M6 20h6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      {/* Text lines appearing */}
      <path d="M14 7h6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" opacity="0.4" />
      <path d="M14 12h7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" opacity="0.7" />
      <path d="M14 17h5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      {/* Sparkle effect */}
      <circle cx="19" cy="4" r="1.2" fill="currentColor" />
    </svg>
  );
}
