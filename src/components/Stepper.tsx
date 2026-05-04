interface StepperProps {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  suffix?: string;
  /** Override the default clamped +/- math, e.g. for cross-field rollover. */
  onIncrement?: () => void;
  onDecrement?: () => void;
  canIncrement?: boolean;
  canDecrement?: boolean;
}

export function Stepper({
  value,
  onChange,
  min = 0,
  max = Infinity,
  step = 1,
  suffix,
  onIncrement,
  onDecrement,
  canIncrement,
  canDecrement,
}: StepperProps) {
  const dec = onDecrement ?? (() => onChange(Math.max(min, value - step)));
  const inc = onIncrement ?? (() => onChange(Math.min(max, value + step)));
  const decDisabled = canDecrement === undefined ? value <= min : !canDecrement;
  const incDisabled = canIncrement === undefined ? value >= max : !canIncrement;

  return (
    <div className="stepper">
      <button
        type="button"
        className="stepper-btn"
        onClick={dec}
        disabled={decDisabled}
        aria-label="Decrease"
      >
        <MinusIcon />
      </button>
      <div className="stepper-value">
        {value}
        {suffix && <span className="stepper-suffix">{suffix}</span>}
      </div>
      <button
        type="button"
        className="stepper-btn"
        onClick={inc}
        disabled={incDisabled}
        aria-label="Increase"
      >
        <PlusIcon />
      </button>
    </div>
  );
}

function MinusIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
      <path d="M2 6H10" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
    </svg>
  );
}

function PlusIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
      <path d="M6 2V10M2 6H10" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
    </svg>
  );
}
