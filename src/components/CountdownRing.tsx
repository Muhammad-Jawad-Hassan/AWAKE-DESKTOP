import { formatClock } from "@/lib/format";

interface CountdownRingProps {
  remainingSecs: number;
  progress: number;
}

const SIZE = 148;
const STROKE = 8;
const RADIUS = (SIZE - STROKE) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export function CountdownRing({ remainingSecs, progress }: CountdownRingProps) {
  const offset = CIRCUMFERENCE * (1 - Math.min(1, Math.max(0, progress)));

  return (
    <div className="countdown-ring">
      <svg width={SIZE} height={SIZE}>
        <circle
          className="countdown-ring-track"
          cx={SIZE / 2}
          cy={SIZE / 2}
          r={RADIUS}
          strokeWidth={STROKE}
          fill="none"
        />
        <circle
          className="countdown-ring-progress"
          cx={SIZE / 2}
          cy={SIZE / 2}
          r={RADIUS}
          strokeWidth={STROKE}
          fill="none"
          strokeDasharray={CIRCUMFERENCE}
          strokeDashoffset={offset}
        />
      </svg>
      <div className="countdown-center">
        <div className="countdown-time">{formatClock(remainingSecs)}</div>
      </div>
    </div>
  );
}
