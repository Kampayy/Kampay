import { Container, SectionHeading } from "@/components/section";

const PRODUCTS = [
  {
    name: "Payroll streams",
    tagline: "For employees and long-term contributors",
    points: [
      "Full cycle funded before it begins, and publicly verifiable",
      "Per-second accrual or fixed weekly, bi-weekly, monthly runs",
      "One transaction pays a 500-person team",
      "Withdraw accrued earnings any day, not just payday",
    ],
    status: "Contracts written",
  },
  {
    name: "Milestone escrow",
    tagline: "For freelancers, agencies and contractors",
    points: [
      "Contract value locked at signing, split per deliverable",
      "Approve milestone one and pay for it while two is in progress",
      "Auto-release after the deadline and grace period lapse",
      "Cancellation and kill-fee terms fixed at signing",
    ],
    status: "Contracts written",
  },
  {
    name: "Credit score",
    tagline: "The record a bank never kept for you",
    points: [
      "Derived from settled payments, disputes and repayments",
      "Two-sided — employers who pay late carry it too",
      "Public inputs and published weights, recomputable by anyone",
      "Lives on Stellar, readable by any protocol",
    ],
    status: "Contracts written",
    highlight: true,
  },
  {
    name: "Lending market",
    tagline: "Where the score becomes money",
    points: [
      "Borrow against history instead of posting 150% collateral",
      "Repayment deducted from payroll at the contract level",
      "Salary advances on earnings already accrued",
      "Open lender pools with risk-tiered yield",
    ],
    status: "Design stage",
  },
];

export function Products() {
  return (
    <section id="products" className="section-wash scroll-mt-20 border-b border-line bg-sunken">
      <Container className="py-16 sm:py-20">
        <SectionHeading
          eyebrow="Products"
          title="Four contracts, one ledger of record"
          lede="Payroll and escrow move the money. Credit reads what they emit. Lending reads credit and writes repayment back into payroll."
        />

        <div className="mt-12 grid gap-5 md:grid-cols-2">
          {PRODUCTS.map((product) => (
            <article
              key={product.name}
              className={`rounded-xl border bg-surface p-6 ${
                product.highlight
                  ? "border-brand/40 ring-1 ring-brand/15"
                  : "border-line"
              }`}
            >
              <div className="flex items-start justify-between gap-3">
                <div>
                  <h3 className="text-lg font-semibold tracking-tight text-ink">
                    {product.name}
                  </h3>
                  <p className="mt-0.5 text-sm text-muted">{product.tagline}</p>
                </div>
                <span className="shrink-0 rounded-full border border-line px-2.5 py-1 text-[11px] whitespace-nowrap text-faint">
                  {product.status}
                </span>
              </div>

              <ul className="mt-5 space-y-2.5">
                {product.points.map((point) => (
                  <li key={point} className="flex gap-2.5 text-sm text-muted">
                    <svg
                      viewBox="0 0 16 16"
                      className="mt-1 h-3.5 w-3.5 shrink-0 fill-brand"
                      aria-hidden="true"
                    >
                      <path d="M6.2 11.3 3.4 8.5l1.1-1.1 1.7 1.7 4.3-4.3 1.1 1.1z" />
                    </svg>
                    <span className="leading-relaxed">{point}</span>
                  </li>
                ))}
              </ul>
            </article>
          ))}
        </div>
      </Container>
    </section>
  );
}
