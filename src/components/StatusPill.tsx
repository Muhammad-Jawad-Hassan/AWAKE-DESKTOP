interface StatusPillProps {
  label: string;
  tone?: "default" | "active" | "success" | "danger";
}

export function StatusPill({ label, tone = "default" }: StatusPillProps) {
  const toneClass =
    tone === "active"
      ? "pill-active"
      : tone === "success"
        ? "pill-success"
        : tone === "danger"
          ? "pill-danger"
          : "";
  return (
    <span className={`pill ${toneClass}`}>
      <span className="pill-dot" />
      {label}
    </span>
  );
}
