interface StepperProps {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  suffix?: string;
  /** Names the control for screen readers, e.g. "mouse moves per minute". */
  label?: string;
  /** Single-line layout for settings rows. */
  inline?: boolean;
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
  label,
  inline,
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
    <div className={`stepper${inline ? " stepper-inline" : ""}`} role="group" aria-label={label}>
      <button
        type="button"
        className="stepper-btn"
        onClick={dec}
        disabled={decDisabled}
        aria-label={label ? `Decrease ${label}` : "Decrease"}
      >
        <MinusIcon />
      </button>
      <div className="stepper-value" aria-live="polite">
        {value}
        {suffix && <span className="stepper-suffix">{suffix}</span>}
      </div>
      <button
        type="button"
        className="stepper-btn"
        onClick={inc}
        disabled={incDisabled}
        aria-label={label ? `Increase ${label}` : "Increase"}
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
