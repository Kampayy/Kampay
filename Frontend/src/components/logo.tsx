export function Logo({ className = "" }: { className?: string }) {
  return (
    <span className={`inline-flex items-center gap-2 ${className}`}>
      <svg
        width="24"
        height="24"
        viewBox="0 0 24 24"
        fill="none"
        aria-hidden="true"
        className="shrink-0"
      >
        {/* Two locked halves releasing a payment stream. */}
        <rect
          x="1.5"
          y="1.5"
          width="21"
          height="21"
          rx="6"
          className="fill-brand"
        />
        <path
          d="M8 6.5v11M8 12l6.5-5.5M8 12l6.5 5.5"
          stroke="var(--brand-ink)"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
      <span className="text-[17px] font-semibold tracking-tight text-ink">
        Kam<span className="text-brand">Pay</span>
      </span>
    </span>
  );
}
