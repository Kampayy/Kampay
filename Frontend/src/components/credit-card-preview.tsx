/**
 * Illustrative credit record, shown on the marketing page to make the scoring
 * model concrete. The weights match `contract/contracts/credit/src/scoring.rs`;
 * the account and its numbers are fictional and labelled as such.
 */

const COMPONENTS = [
  { label: "Payment history", weight: 35, score: 940 },
  { label: "Tenure", weight: 20, score: 720 },
  { label: "Income consistency", weight: 15, score: 830 },
  { label: "Counterparty quality", weight: 10, score: 690 },
  { label: "Dispute record", weight: 10, score: 1000 },
  { label: "Loan repayment", weight: 10, score: 800 },
];

const TOTAL = Math.round(
  COMPONENTS.reduce((sum, c) => sum + c.score * c.weight, 0) / 100,
);

export function CreditCardPreview() {
  return (
    <figure className="w-full rounded-xl border border-line bg-surface p-5 shadow-[var(--shadow-lift)]">
      <figcaption className="flex items-baseline justify-between gap-3">
        <span className="text-xs font-semibold uppercase tracking-wider text-faint">
          Credit record
        </span>
        <span className="rounded-full border border-line px-2 py-0.5 text-[11px] text-faint">
          Example
        </span>
      </figcaption>

      <p className="mt-1 font-mono text-xs text-faint">GDRX…7QK4</p>

      <div className="mt-4 flex items-end gap-3">
        <span className="font-mono text-5xl font-semibold leading-none tracking-tight text-ink tabular-nums">
          {TOTAL}
        </span>
        <span className="pb-1 font-mono text-sm text-faint">/ 1000</span>
        <span className="ml-auto mb-1 rounded-md bg-accent-wash px-2 py-1 text-xs font-medium text-accent">
          Gold tier
        </span>
      </div>

      <ul className="mt-6 space-y-3.5">
        {COMPONENTS.map((component) => (
          <li key={component.label}>
            <div className="flex items-baseline justify-between gap-3">
              <span className="flex items-baseline gap-2 text-xs text-muted">
                {component.label}
                <span className="font-mono text-[10px] text-faint">
                  {component.weight}%
                </span>
              </span>
              <span className="font-mono text-xs text-ink tabular-nums">
                {component.score}
              </span>
            </div>
            <div className="mt-1.5 h-1.5 overflow-hidden rounded-full bg-sunken">
              <div
                className="h-full rounded-full bg-brand"
                style={{ width: `${(component.score / 1000) * 100}%` }}
              />
            </div>
          </li>
        ))}
      </ul>

      <div className="mt-5 rounded-lg border border-brand/25 bg-brand-wash p-3">
        <p className="text-xs text-muted">Unlocks at this score</p>
        <p className="mt-0.5 text-sm font-medium text-ink">
          Up to{" "}
          <span className="font-mono tabular-nums">12,000 USDC</span> —
          under-collateralised, repaid from payroll
        </p>
      </div>
    </figure>
  );
}
