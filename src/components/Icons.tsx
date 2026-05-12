type IconProps = { size?: number };

export function BackIcon({ size = 16 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path
        d="M9.5 3.5L4.5 8L9.5 12.5"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function GearIcon({ size = 16 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path
        d="M3 4.5h10M3 8h10M3 11.5h10"
        stroke="currentColor"
        strokeWidth="1.4"
        strokeLinecap="round"
      />
      <circle cx="6" cy="4.5" r="1.5" fill="currentColor" />
      <circle cx="10.5" cy="8" r="1.5" fill="currentColor" />
      <circle cx="7" cy="11.5" r="1.5" fill="currentColor" />
    </svg>
  );
}

export function PowerIcon({ size = 16 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M8 2.2V7.6" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
      <path
        d="M11.6 3.9A5.2 5.2 0 1 1 4.4 3.9"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function StopIcon({ size = 14 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <rect x="3.5" y="3.5" width="9" height="9" rx="1.5" fill="currentColor" />
    </svg>
  );
}

export function PauseIcon({ size = 14 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <rect x="4" y="3.5" width="2.6" height="9" rx="0.8" fill="currentColor" />
      <rect x="9.4" y="3.5" width="2.6" height="9" rx="0.8" fill="currentColor" />
    </svg>
  );
}

export function PlayIcon({ size = 14 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M4.5 3.2v9.6l8-4.8-8-4.8z" fill="currentColor" />
    </svg>
  );
}

export function AlertIcon({ size = 14 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path
        d="M8 1.8 14.6 13a1 1 0 0 1-.86 1.5H2.26A1 1 0 0 1 1.4 13L8 1.8z"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinejoin="round"
      />
      <path d="M8 6.2v3.4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
      <circle cx="8" cy="11.6" r="0.9" fill="currentColor" />
    </svg>
  );
}

export function ChevronRightIcon({ size = 14 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path
        d="M6 3.5L11 8L6 12.5"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
