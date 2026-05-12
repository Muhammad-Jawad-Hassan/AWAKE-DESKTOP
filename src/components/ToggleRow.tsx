import { Toggle } from "@/components/Toggle";

interface ToggleRowProps {
  label: string;
  hint?: string;
  checked: boolean;
  disabled?: boolean;
  onChange: (checked: boolean) => void;
}

export function ToggleRow({ label, hint, checked, disabled, onChange }: ToggleRowProps) {
  return (
    <div className="row">
      <div>
        <div className="field-label">{label}</div>
        {hint && <div className="field-hint">{hint}</div>}
      </div>
      <Toggle checked={checked} onChange={onChange} disabled={disabled} />
    </div>
  );
}
